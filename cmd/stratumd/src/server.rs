use anyhow::Result;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::*;
use starcoin_stratumd::codec::{JsonStreamCodec, Separator};
use starcoin_stratumd::stratum_rpc::{LoginRequest, ShareRequest, Status, SubmitShareResponse};
use starcoin_stratumd::{
    MAX_AGENT_LEN, MAX_LOGIN_LEN, MAX_PASS_LEN, MAX_PROTOCOL_ERRORS, MAX_REQS_PER_WINDOW,
    OUTBOUND_CHANNEL_CAP, PROTOCOL_ERROR_WINDOW_SECS, READ_IDLE_TIMEOUT_SECS, REQ_WINDOW_SECS,
    WRITE_DRAIN_TIMEOUT_SECS,
};
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio_util::codec::Framed;

use crate::gateway::App;

#[derive(Debug, Clone)]
struct RequestRate {
    first: Instant,
    count: u32,
}

impl RequestRate {
    fn new() -> Self {
        Self {
            first: Instant::now(),
            count: 0,
        }
    }

    fn exceeded(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.first) > Duration::from_secs(REQ_WINDOW_SECS) {
            self.first = now;
            self.count = 1;
            return false;
        }
        self.count = self.count.saturating_add(1);
        self.count > MAX_REQS_PER_WINDOW
    }
}

#[derive(Debug, Clone)]
struct ProtocolErrorCounter {
    first: Instant,
    count: u32,
}

impl ProtocolErrorCounter {
    fn new() -> Self {
        Self {
            first: Instant::now(),
            count: 0,
        }
    }

    fn record_and_exceeded(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.first) > Duration::from_secs(PROTOCOL_ERROR_WINDOW_SECS) {
            self.first = now;
            self.count = 1;
            return false;
        }
        self.count = self.count.saturating_add(1);
        self.count >= MAX_PROTOCOL_ERRORS
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    #[serde(default)]
    jsonrpc: Option<String>,
    #[serde(default)]
    id: Option<JsonRpcId>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
enum JsonRpcId {
    Number(u64),
    String(String),
}

#[derive(Debug, Serialize)]
struct JsonRpcOutput<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    jsonrpc: Option<&'static str>,
    result: T,
    id: JsonRpcId,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcFailure {
    #[serde(skip_serializing_if = "Option::is_none")]
    jsonrpc: Option<&'static str>,
    id: JsonRpcId,
    error: JsonRpcError,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

fn parse_request_id(id: Option<JsonRpcId>) -> Option<JsonRpcId> {
    id
}

fn parse_params<T: DeserializeOwned>(params: serde_json::Value) -> Result<T> {
    serde_json::from_value(params).map_err(|err| anyhow::anyhow!("invalid params: {}", err))
}

fn send_output<T: Serialize>(
    out_tx: &mpsc::Sender<String>,
    id: JsonRpcId,
    result: T,
) -> Result<()> {
    let output = JsonRpcOutput {
        jsonrpc: Some("2.0"),
        result,
        id,
        error: None,
    };
    let msg = serde_json::to_string(&output)?;
    try_send_msg(out_tx, msg)
}

fn send_failure(
    out_tx: &mpsc::Sender<String>,
    id: JsonRpcId,
    code: i32,
    message: String,
) -> Result<()> {
    let failure = JsonRpcFailure {
        jsonrpc: Some("2.0"),
        id,
        error: JsonRpcError { code, message },
    };
    let msg = serde_json::to_string(&failure)?;
    try_send_msg(out_tx, msg)
}

fn try_send_msg(out_tx: &mpsc::Sender<String>, msg: String) -> Result<()> {
    let mut out_tx = out_tx.clone();
    match out_tx.try_send(msg) {
        Ok(()) => Ok(()),
        Err(err) => Err(anyhow::anyhow!("send response failed: {}", err)),
    }
}

pub async fn run_stratum_server(address: SocketAddr, app: App) -> Result<()> {
    let listener = TcpListener::bind(address).await?;
    info!(target: "stratum_server", "Stratum tcp server start at: {}", address);

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!(target: "stratum_server", "stratum client connected: {}", peer_addr);
                let app = app.clone();
                tokio::spawn(async move {
                    handle_connection(stream, app).await;
                });
            }
            Err(err) => {
                error!(target: "stratum_server", "accept connection failed: {}", err);
            }
        }
    }
}

async fn handle_connection(stream: TcpStream, app: App) {
    let peer_addr = stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let framed = Framed::new(
        stream,
        JsonStreamCodec::new(Separator::Byte(b'\n'), Default::default()),
    );
    let (mut sink, mut stream) = framed.split();
    let (out_tx, mut out_rx) = mpsc::channel::<String>(OUTBOUND_CHANNEL_CAP);
    let mut logged_in = false;
    let mut worker_id: Option<String> = None;
    let mut req_rate = RequestRate::new();
    let mut protocol_errors = ProtocolErrorCounter::new();
    let mut disconnect_reason: Option<String> = None;

    let writer_peer = peer_addr.clone();
    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.next().await {
            if sink.send(msg).await.is_err() {
                debug!(
                    target: "stratum_server",
                    "disconnect client: peer={}, reason=write failed",
                    writer_peer
                );
                break;
            }
        }
    });

    loop {
        let item = match timeout(Duration::from_secs(READ_IDLE_TIMEOUT_SECS), stream.next()).await {
            Ok(item) => item,
            Err(_) => {
                disconnect_reason = Some("read timeout".to_string());
                break;
            }
        };

        let item = match item {
            Some(item) => item,
            None => {
                disconnect_reason = Some("client closed".to_string());
                break;
            }
        };

        let line = match item {
            Ok(line) => line,
            Err(err) => {
                disconnect_reason = Some(format!("read error: {err}"));
                break;
            }
        };

        debug!(target: "stratum_server", "recv line: {}", line);

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(err) => {
                disconnect_reason = Some(format!("invalid jsonrpc request: {err}"));
                break;
            }
        };

        let request_id = parse_request_id(request.id);
        if request_id.is_none() {
            disconnect_reason = Some("missing request id".to_string());
            break;
        }

        if request.method != "submit" && req_rate.exceeded() {
            disconnect_reason = Some("request rate limit exceeded".to_string());
            break;
        }

        match request.method.as_str() {
            "login" => {
                if logged_in {
                    if let Some(id) = request_id {
                        let _ = send_failure(&out_tx, id, -1, "duplicate login".to_string());
                    }
                    disconnect_reason = Some("duplicate login".to_string());
                    break;
                }

                let login: LoginRequest = match parse_params(request.params) {
                    Ok(login) => login,
                    Err(err) => {
                        if let Some(id) = request_id {
                            let _ = send_failure(&out_tx, id, -1, err.to_string());
                        }
                        disconnect_reason = Some(format!("invalid login params: {err}"));
                        break;
                    }
                };

                if let Err(err) = validate_login_request(&login) {
                    if let Some(id) = request_id {
                        let _ = send_failure(&out_tx, id, -1, err.to_string());
                    }
                    disconnect_reason = Some(format!("invalid login request: {err}"));
                    break;
                }

                match app.register_worker(login, out_tx.clone()).await {
                    Ok((wid, first_job)) => {
                        if let Some(id) = request_id {
                            if let Err(err) = send_output(&out_tx, id, first_job) {
                                disconnect_reason =
                                    Some(format!("send login response failed: {err}"));
                                break;
                            }
                        }
                        logged_in = true;
                        worker_id = Some(wid);
                    }
                    Err(err) => {
                        if let Some(id) = request_id {
                            let _ = send_failure(&out_tx, id, -1, err.to_string());
                        }
                        disconnect_reason = Some(format!("handle login failed: {err}"));
                        break;
                    }
                }
            }
            "submit" => {
                if !logged_in {
                    if let Some(id) = request_id {
                        let _ = send_failure(&out_tx, id, -1, "submit before login".to_string());
                    }
                    disconnect_reason = Some("submit before login".to_string());
                    break;
                }

                let share: ShareRequest = match parse_params(request.params) {
                    Ok(share) => share,
                    Err(err) => {
                        if let Some(id) = request_id {
                            let _ = send_failure(&out_tx, id, -1, err.to_string());
                        }
                        if protocol_errors.record_and_exceeded() {
                            disconnect_reason = Some(
                                "protocol error threshold exceeded (invalid params)".to_string(),
                            );
                            break;
                        }
                        continue;
                    }
                };

                if let Some(expected) = worker_id.as_ref() {
                    if share.id != *expected {
                        if let Some(id) = request_id {
                            let _ = send_failure(&out_tx, id, -1, "worker mismatch".to_string());
                        }
                        if protocol_errors.record_and_exceeded() {
                            disconnect_reason = Some(
                                "protocol error threshold exceeded (worker mismatch)".to_string(),
                            );
                            break;
                        }
                        continue;
                    }
                }

                match app.submit_share(share).await {
                    SubmitShareResponse::Accepted => {
                        if let Some(id) = request_id {
                            let status = Status {
                                status: "OK".to_string(),
                            };
                            if let Err(err) = send_output(&out_tx, id, status) {
                                disconnect_reason =
                                    Some(format!("send submit success response failed: {err}"));
                                break;
                            }
                        }
                    }
                    SubmitShareResponse::Rejected {
                        code,
                        message,
                        disconnect,
                    } => {
                        if let Some(id) = request_id {
                            let _ = send_failure(&out_tx, id, code, message.clone());
                        }
                        if disconnect {
                            disconnect_reason =
                                Some(format!("share rejected (disconnect): {message}"));
                            break;
                        }
                    }
                }
            }
            "keepalived" => {
                if !logged_in {
                    if let Some(id) = request_id {
                        let _ = send_failure(&out_tx, id, -1, "keepalive before login".to_string());
                    }
                    disconnect_reason = Some("keepalive before login".to_string());
                    break;
                }
                if let Some(id) = request_id {
                    let status = Status {
                        status: "KEEPALIVED".to_string(),
                    };
                    if let Err(err) = send_output(&out_tx, id, status) {
                        disconnect_reason = Some(format!("send keepalive response failed: {err}"));
                        break;
                    }
                }
            }
            "logout" => {
                if !logged_in {
                    if let Some(id) = request_id {
                        let _ = send_failure(&out_tx, id, -1, "logout before login".to_string());
                    }
                    disconnect_reason = Some("logout before login".to_string());
                    break;
                }
                if let Some(id) = request_id {
                    let _ = send_output(&out_tx, id, false);
                }
                break;
            }
            _ => {
                if let Some(id) = request_id {
                    let _ = send_failure(&out_tx, id, -1, "method not found".to_string());
                }
                if protocol_errors.record_and_exceeded() {
                    disconnect_reason =
                        Some("protocol error threshold exceeded (method not found)".to_string());
                    break;
                }
            }
        }
    }

    drop(out_tx);
    let _ = timeout(Duration::from_secs(WRITE_DRAIN_TIMEOUT_SECS), writer).await;

    if let Some(reason) = disconnect_reason {
        if !logged_in && reason == "client closed" {
            debug!(
                target: "stratum_server",
                "disconnect client: peer={}, worker={}, logged_in={}, reason={}",
                peer_addr,
                worker_id.clone().unwrap_or_else(|| "-".to_string()),
                logged_in,
                reason
            );
        } else {
            warn!(
                target: "stratum_server",
                "disconnect client: peer={}, worker={}, logged_in={}, reason={}",
                peer_addr,
                worker_id.clone().unwrap_or_else(|| "-".to_string()),
                logged_in,
                reason
            );
        }
    }

    if let Some(worker_id) = worker_id {
        app.unregister_worker_hex(&worker_id).await;
    }
}

fn validate_login_request(login: &LoginRequest) -> Result<()> {
    if login.login.trim().is_empty() {
        return Err(anyhow::anyhow!("login is empty"));
    }
    if login.login.len() > MAX_LOGIN_LEN {
        return Err(anyhow::anyhow!("login too long"));
    }
    if login.pass.len() > MAX_PASS_LEN {
        return Err(anyhow::anyhow!("pass too long"));
    }
    if login.agent.len() > MAX_AGENT_LEN {
        return Err(anyhow::anyhow!("agent too long"));
    }
    Ok(())
}
