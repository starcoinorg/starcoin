use crate::codec::JsonStreamCodec;
use crate::rpc::{
    LoginRequest, ShareRequest, Status, SubmitShareEvent, SubmitShareResponse, SubscribeJobEvent,
    UnsubscribeWorkerEvent, WorkerId,
};
use crate::stratum::Stratum;
use anyhow::Result;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory, ServiceRef};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};
use tokio_util::codec::Framed;

const OUTBOUND_CHANNEL_CAP: usize = 128;
const MAX_LOGIN_LEN: usize = 256;
const MAX_PASS_LEN: usize = 256;
const MAX_AGENT_LEN: usize = 128;
const READ_IDLE_TIMEOUT_SECS: u64 = 600;
const REQ_WINDOW_SECS: u64 = 10;
const MAX_REQS_PER_WINDOW: u32 = 100;

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

pub struct StratumService {
    config: Arc<NodeConfig>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    join_handle: Option<JoinHandle<()>>,
}

impl ActorService for StratumService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        if let Some(address) = self.config.stratum.get_address() {
            let stratum = ctx.service_ref::<Stratum>()?.clone();
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            let join_handle = std::thread::spawn(move || {
                let runtime = Runtime::new().expect("create stratum tokio runtime");
                let result = runtime.block_on(run_stratum_server(address, stratum, shutdown_rx));
                if let Err(err) = result {
                    error!(target: "stratum", "stratum server stopped with error: {}", err);
                }
            });
            self.shutdown_tx = Some(shutdown_tx);
            self.join_handle = Some(join_handle);
            info!(target: "stratum", "Stratum tcp server start at: {}", address);
        }
        Ok(())
    }

    fn stopped(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
        Ok(())
    }
}

pub struct StratumServiceFactory;

impl ServiceFactory<StratumService> for StratumServiceFactory {
    fn create(ctx: &mut ServiceContext<StratumService>) -> Result<StratumService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        Ok(StratumService {
            config,
            shutdown_tx: None,
            join_handle: None,
        })
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

#[derive(Debug, Serialize)]
struct JsonRpcNotification<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    jsonrpc: Option<&'static str>,
    method: &'static str,
    params: T,
}

async fn run_stratum_server(
    address: std::net::SocketAddr,
    stratum: ServiceRef<Stratum>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let listener = TcpListener::bind(address).await?;
    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                break;
            }
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, peer_addr)) => {
                        info!(target: "stratum", "stratum client connected: {}", peer_addr);
                        tokio::spawn(handle_connection(stream, stratum.clone()));
                    }
                    Err(err) => {
                        error!(target: "stratum", "accept connection failed: {}", err);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_connection(stream: TcpStream, stratum: ServiceRef<Stratum>) {
    let framed = Framed::new(stream, JsonStreamCodec::stream_incoming());
    let (mut sink, mut stream) = framed.split();
    let (out_tx, mut out_rx) = mpsc::channel::<String>(OUTBOUND_CHANNEL_CAP);
    let mut logged_in = false;
    let mut worker_id: Option<String> = None;
    let mut req_rate = RequestRate::new();

    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.next().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    loop {
        let item = match timeout(Duration::from_secs(READ_IDLE_TIMEOUT_SECS), stream.next()).await {
            Ok(item) => item,
            Err(_) => {
                debug!(target: "stratum", "stratum read timeout");
                break;
            }
        };
        let item = match item {
            Some(item) => item,
            None => break,
        };
        let line = match item {
            Ok(line) => line,
            Err(err) => {
                debug!(target: "stratum", "stratum read error: {}", err);
                break;
            }
        };
        if req_rate.exceeded() {
            debug!(target: "stratum", "request rate limit exceeded");
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(err) => {
                debug!(target: "stratum", "invalid jsonrpc request: {}", err);
                break;
            }
        };
        let request_id = parse_request_id(request.id);
        if request_id.is_none() {
            debug!(target: "stratum", "missing request id");
            break;
        }
        match request.method.as_str() {
            "login" => {
                if logged_in {
                    let _ = send_failure(
                        &out_tx,
                        request_id.clone().unwrap(),
                        -1,
                        "duplicate login".into(),
                    );
                    break;
                }
                match handle_login(request_id, request.params, &stratum, &out_tx).await {
                    Ok(LoginAction::Continue { worker_id: id }) => {
                        logged_in = true;
                        worker_id = Some(id);
                    }
                    Ok(LoginAction::Disconnect) => break,
                    Err(err) => {
                        debug!(target: "stratum", "handle login failed: {}", err);
                        break;
                    }
                }
            }
            "submit" => {
                if !logged_in {
                    let _ = send_failure(
                        &out_tx,
                        request_id.clone().unwrap(),
                        -1,
                        "not logged in".into(),
                    );
                    break;
                }
                match handle_submit(
                    request_id,
                    request.params,
                    &stratum,
                    &out_tx,
                    worker_id.as_deref(),
                )
                .await
                {
                    Ok(SubmitAction::Continue) => {}
                    Ok(SubmitAction::Disconnect) => break,
                    Err(err) => {
                        debug!(target: "stratum", "handle submit failed: {}", err);
                        break;
                    }
                }
            }
            "keepalived" => {
                let id = request_id.clone().unwrap();
                let status = Status {
                    status: "KEEPALIVED".to_string(),
                };
                if let Err(err) = send_output(&out_tx, id, status) {
                    debug!(target: "stratum", "send keepalived response failed: {}", err);
                    break;
                }
            }
            "logout" => {
                if !logged_in {
                    break;
                }
                if let Err(err) = handle_logout(request_id, request.params, &out_tx).await {
                    debug!(target: "stratum", "handle logout failed: {}", err);
                }
                break;
            }
            other => {
                let id = request_id.clone().unwrap();
                let _ = send_failure(&out_tx, id, -1, format!("unknown method {}", other));
                break;
            }
        }
    }

    writer.abort();

    if let Some(worker_id) = worker_id {
        if let Ok(worker_id) = WorkerId::from_hex(worker_id) {
            let _ = stratum.send(UnsubscribeWorkerEvent { worker_id }).await;
        }
    }
}

enum LoginAction {
    Continue { worker_id: String },
    Disconnect,
}

async fn handle_login(
    request_id: Option<JsonRpcId>,
    params: serde_json::Value,
    stratum: &ServiceRef<Stratum>,
    out_tx: &mpsc::Sender<String>,
) -> Result<LoginAction> {
    let login: LoginRequest = match parse_params(params) {
        Ok(login) => login,
        Err(err) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            return Ok(LoginAction::Disconnect);
        }
    };
    if let Err(err) = validate_login_request(&login) {
        if let Some(id) = request_id {
            let _ = send_failure(out_tx, id, -1, err.to_string());
        }
        return Ok(LoginAction::Disconnect);
    }
    let mut job_rx = match stratum.send(SubscribeJobEvent(login)).await {
        Ok(Ok(rx)) => rx,
        Ok(Err(err)) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            return Ok(LoginAction::Disconnect);
        }
        Err(err) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            return Ok(LoginAction::Disconnect);
        }
    };

    if let Some(id) = request_id {
        let mut first_job = match job_rx.next().await {
            Some(job) => job,
            None => {
                let _ = send_failure(out_tx, id, -1, "no job".to_string());
                return Ok(LoginAction::Disconnect);
            }
        };
        let worker_id = first_job.id.clone();
        first_job.login = None;
        send_output(out_tx, id, first_job)?;

        let out_tx = out_tx.clone();
        tokio::spawn(async move {
            while let Some(job_resp) = job_rx.next().await {
                let notif = JsonRpcNotification {
                    jsonrpc: Some("2.0"),
                    method: "job",
                    params: job_resp.job,
                };
                let msg = match serde_json::to_string(&notif) {
                    Ok(msg) => msg,
                    Err(err) => {
                        debug!(target: "stratum", "serialize job notification failed: {}", err);
                        continue;
                    }
                };
                if try_send_msg(&out_tx, msg).is_err() {
                    break;
                }
            }
        });

        return Ok(LoginAction::Continue { worker_id });
    }

    Ok(LoginAction::Disconnect)
}

enum SubmitAction {
    Continue,
    Disconnect,
}

async fn handle_submit(
    request_id: Option<JsonRpcId>,
    params: serde_json::Value,
    stratum: &ServiceRef<Stratum>,
    out_tx: &mpsc::Sender<String>,
    expected_worker_id: Option<&str>,
) -> Result<SubmitAction> {
    let share: ShareRequest = match parse_params(params) {
        Ok(share) => share,
        Err(err) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            return Ok(SubmitAction::Disconnect);
        }
    };
    if let Some(expected) = expected_worker_id {
        if share.id != expected {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, "worker mismatch".to_string());
            }
            return Ok(SubmitAction::Disconnect);
        }
    }
    let submit_result = stratum.send(SubmitShareEvent(share)).await;
    match submit_result {
        Ok(Ok(SubmitShareResponse::Accepted)) => {
            if let Some(id) = request_id {
                let status = Status {
                    status: "OK".to_string(),
                };
                if send_output(out_tx, id, status).is_err() {
                    return Ok(SubmitAction::Disconnect);
                }
            }
            Ok(SubmitAction::Continue)
        }
        Ok(Ok(SubmitShareResponse::Rejected {
            code,
            message,
            disconnect,
        })) => {
            if let Some(id) = request_id {
                if send_failure(out_tx, id, code, message).is_err() {
                    return Ok(SubmitAction::Disconnect);
                }
            }
            if disconnect {
                Ok(SubmitAction::Disconnect)
            } else {
                Ok(SubmitAction::Continue)
            }
        }
        Ok(Err(err)) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            Ok(SubmitAction::Disconnect)
        }
        Err(err) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            Ok(SubmitAction::Disconnect)
        }
    }
}

async fn handle_logout(
    request_id: Option<JsonRpcId>,
    params: serde_json::Value,
    out_tx: &mpsc::Sender<String>,
) -> Result<()> {
    info!(target: "stratum", "receive logout request params: {}", params);
    if let Some(id) = request_id {
        let _ = send_output(out_tx, id, false);
    }
    Ok(())
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
