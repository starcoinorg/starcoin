use crate::codec::JsonStreamCodec;
use crate::rpc::{LoginRequest, ShareRequest, Status, SubmitShareEvent, SubscribeJobEvent};
use crate::stratum::Stratum;
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, ServiceContext, ServiceFactory, ServiceRef};
use std::sync::Arc;
use std::thread::JoinHandle;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio_util::codec::Framed;

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

#[derive(Debug, Deserialize)]
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
    id: u32,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcFailure {
    #[serde(skip_serializing_if = "Option::is_none")]
    jsonrpc: Option<&'static str>,
    id: u32,
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
    let (out_tx, mut out_rx) = futures::channel::mpsc::unbounded::<String>();

    let writer = tokio::spawn(async move {
        while let Some(msg) = out_rx.next().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(item) = stream.next().await {
        let line = match item {
            Ok(line) => line,
            Err(err) => {
                debug!(target: "stratum", "stratum read error: {}", err);
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(err) => {
                debug!(target: "stratum", "invalid jsonrpc request: {}", err);
                continue;
            }
        };
        let request_id = parse_request_id(request.id);
        match request.method.as_str() {
            "login" => {
                if let Err(err) = handle_login(request_id, request.params, &stratum, &out_tx).await
                {
                    debug!(target: "stratum", "handle login failed: {}", err);
                }
            }
            "submit" => {
                if let Err(err) = handle_submit(request_id, request.params, &stratum, &out_tx).await
                {
                    debug!(target: "stratum", "handle submit failed: {}", err);
                }
            }
            "keepalived" => {
                if let Some(id) = request_id {
                    let status = Status {
                        status: "KEEPALIVED".to_string(),
                    };
                    if let Err(err) = send_output(&out_tx, id, status) {
                        debug!(target: "stratum", "send keepalived response failed: {}", err);
                    }
                }
            }
            "logout" => {
                if let Err(err) = handle_logout(request_id, request.params, &out_tx).await {
                    debug!(target: "stratum", "handle logout failed: {}", err);
                }
            }
            other => {
                if let Some(id) = request_id {
                    let _ = send_failure(&out_tx, id, -1, format!("unknown method {}", other));
                }
            }
        }
    }

    writer.abort();
}

async fn handle_login(
    request_id: Option<u32>,
    params: serde_json::Value,
    stratum: &ServiceRef<Stratum>,
    out_tx: &futures::channel::mpsc::UnboundedSender<String>,
) -> Result<()> {
    let login: LoginRequest = match parse_params(params) {
        Ok(login) => login,
        Err(err) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            return Ok(());
        }
    };
    let mut job_rx = match stratum.send(SubscribeJobEvent(login)).await {
        Ok(Ok(rx)) => rx,
        Ok(Err(err)) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            return Ok(());
        }
        Err(err) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            return Ok(());
        }
    };

    if let Some(id) = request_id {
        let mut first_job = match job_rx.next().await {
            Some(job) => job,
            None => {
                let _ = send_failure(out_tx, id, -1, "no job".to_string());
                return Ok(());
            }
        };
        first_job.login = None;
        send_output(out_tx, id, first_job)?;
    }

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
            if out_tx.unbounded_send(msg).is_err() {
                break;
            }
        }
    });

    Ok(())
}

async fn handle_submit(
    request_id: Option<u32>,
    params: serde_json::Value,
    stratum: &ServiceRef<Stratum>,
    out_tx: &futures::channel::mpsc::UnboundedSender<String>,
) -> Result<()> {
    let share: ShareRequest = match parse_params(params) {
        Ok(share) => share,
        Err(err) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
            return Ok(());
        }
    };
    let submit_result = stratum.send(SubmitShareEvent(share)).await;
    match submit_result {
        Ok(Ok(())) => {
            if let Some(id) = request_id {
                let status = Status {
                    status: "OK".to_string(),
                };
                let _ = send_output(out_tx, id, status);
            }
        }
        Ok(Err(err)) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
        }
        Err(err) => {
            if let Some(id) = request_id {
                let _ = send_failure(out_tx, id, -1, err.to_string());
            }
        }
    }
    Ok(())
}

async fn handle_logout(
    request_id: Option<u32>,
    params: serde_json::Value,
    out_tx: &futures::channel::mpsc::UnboundedSender<String>,
) -> Result<()> {
    info!(target: "stratum", "receive logout request params: {}", params);
    if let Some(id) = request_id {
        let _ = send_output(out_tx, id, false);
    }
    Ok(())
}

fn parse_request_id(id: Option<JsonRpcId>) -> Option<u32> {
    match id {
        Some(JsonRpcId::Number(num)) => u32::try_from(num).ok(),
        Some(JsonRpcId::String(s)) => s.parse::<u32>().ok(),
        None => None,
    }
}

fn parse_params<T: DeserializeOwned>(params: serde_json::Value) -> Result<T> {
    serde_json::from_value(params).map_err(|err| anyhow::anyhow!("invalid params: {}", err))
}

fn send_output<T: Serialize>(
    out_tx: &futures::channel::mpsc::UnboundedSender<String>,
    id: u32,
    result: T,
) -> Result<()> {
    let output = JsonRpcOutput {
        jsonrpc: Some("2.0"),
        result,
        id,
        error: None,
    };
    let msg = serde_json::to_string(&output)?;
    out_tx
        .unbounded_send(msg)
        .map_err(|_| anyhow::anyhow!("send response failed"))
}

fn send_failure(
    out_tx: &futures::channel::mpsc::UnboundedSender<String>,
    id: u32,
    code: i32,
    message: String,
) -> Result<()> {
    let failure = JsonRpcFailure {
        jsonrpc: Some("2.0"),
        id,
        error: JsonRpcError { code, message },
    };
    let msg = serde_json::to_string(&failure)?;
    out_tx
        .unbounded_send(msg)
        .map_err(|_| anyhow::anyhow!("send response failed"))
}
