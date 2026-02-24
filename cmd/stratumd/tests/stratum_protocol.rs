use anyhow::{Context, Result};
use jsonrpc_core::{Error as JsonRpcError, IoHandler, Params};
use jsonrpc_ws_server::{Server as WsServer, ServerBuilder};
use once_cell::sync::Lazy;
use serde_json::json;
use starcoin_stratumd::stratum_rpc::{StratumJob, StratumJobResponse};
use starcoin_types::genesis_config::ConsensusStrategy;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::U256;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Instant;

static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Clone, Debug)]
struct SubmitCall {
    minting_blob: String,
    nonce: u32,
    extra: String,
}

#[derive(Clone)]
struct MockRpcState {
    current_job: MintBlockEvent,
    submits: Vec<SubmitCall>,
}

struct MockMiningRpc {
    state: Arc<StdMutex<MockRpcState>>,
    server: Option<WsServer>,
    addr: SocketAddr,
}

impl MockMiningRpc {
    fn start(initial_job: MintBlockEvent) -> Result<Self> {
        let state = Arc::new(StdMutex::new(MockRpcState {
            current_job: initial_job,
            submits: Vec::new(),
        }));

        let mut io = IoHandler::default();

        let state_for_get_job = state.clone();
        io.add_sync_method("mining.get_job", move |_params: Params| {
            let guard = state_for_get_job
                .lock()
                .map_err(|_| JsonRpcError::internal_error())?;
            serde_json::to_value(Some(guard.current_job.clone()))
                .map_err(|_| JsonRpcError::internal_error())
        });

        let state_for_submit = state.clone();
        io.add_sync_method("mining.submit", move |params: Params| {
            let (minting_blob, nonce, extra): (String, u32, String) = params
                .parse()
                .map_err(|_| JsonRpcError::invalid_params("invalid submit params"))?;

            let mut guard = state_for_submit
                .lock()
                .map_err(|_| JsonRpcError::internal_error())?;
            guard.submits.push(SubmitCall {
                minting_blob,
                nonce,
                extra,
            });

            let block_hash = guard.current_job.parent_hash;
            Ok(json!({ "block_hash": block_hash }))
        });

        let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
        let server = ServerBuilder::new(io)
            .start(&addr)
            .context("start mock mining rpc ws server failed")?;

        Ok(Self {
            state,
            server: Some(server),
            addr,
        })
    }

    fn ws_url(&self) -> String {
        format!("ws://{}", self.addr)
    }

    fn set_job(&self, event: MintBlockEvent) -> Result<()> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("mock rpc mutex poisoned"))?;
        guard.current_job = event;
        Ok(())
    }

    fn submit_calls(&self) -> Result<Vec<SubmitCall>> {
        let guard = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("mock rpc mutex poisoned"))?;
        Ok(guard.submits.clone())
    }
}

impl Drop for MockMiningRpc {
    fn drop(&mut self) {
        if let Some(server) = self.server.take() {
            std::mem::forget(server);
        }
    }
}

struct StratumdProcess {
    child: Child,
}

impl StratumdProcess {
    async fn spawn(listen: SocketAddr, node_rpc: &str, extra_args: &[&str]) -> Result<Self> {
        let bin = resolve_stratumd_bin()?;

        let mut cmd = Command::new(&bin);
        cmd.arg("--listen")
            .arg(listen.to_string())
            .arg("--node-rpc")
            .arg(node_rpc)
            .arg("--job-poll-ms")
            .arg("50")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for arg in extra_args {
            cmd.arg(arg);
        }

        let mut child = cmd.spawn().context("spawn stratumd failed")?;
        wait_for_server_ready(&mut child, listen, Duration::from_secs(6)).await?;
        Ok(Self { child })
    }
}

fn resolve_stratumd_bin() -> Result<PathBuf> {
    if let Ok(bin) = std::env::var("CARGO_BIN_EXE_starcoin_stratumd") {
        return Ok(PathBuf::from(bin));
    }

    let current = std::env::current_exe().context("resolve current test executable failed")?;
    let debug_dir = current.parent().and_then(|p| p.parent()).ok_or_else(|| {
        anyhow::anyhow!("cannot locate target/debug directory from {:?}", current)
    })?;
    let bin_name = if cfg!(windows) {
        "starcoin_stratumd.exe"
    } else {
        "starcoin_stratumd"
    };
    let candidate = debug_dir.join(bin_name);
    if candidate.exists() {
        return Ok(candidate);
    }

    Err(anyhow::anyhow!(
        "cannot find stratumd binary via env var or {:?}",
        candidate
    ))
}

impl Drop for StratumdProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn pick_free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    Ok(listener.local_addr()?.port())
}

fn build_mint_event(number: u64, difficulty: u64, strategy: ConsensusStrategy) -> MintBlockEvent {
    let mut minting_blob = vec![0u8; 76];
    minting_blob[0..8].copy_from_slice(&number.to_le_bytes());
    minting_blob[8..16].copy_from_slice(&difficulty.to_le_bytes());
    MintBlockEvent::new(
        starcoin_crypto::HashValue::random(),
        strategy,
        minting_blob,
        U256::from(difficulty.max(1)),
        number,
        None,
    )
}

async fn wait_for_server_ready(
    child: &mut Child,
    addr: SocketAddr,
    timeout: Duration,
) -> Result<()> {
    let start = Instant::now();
    loop {
        if let Ok(stream) = TcpStream::connect(addr).await {
            drop(stream);
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            return Err(anyhow::anyhow!(
                "stratumd exited before ready, status: {}",
                status
            ));
        }
        if start.elapsed() > timeout {
            return Err(anyhow::anyhow!("wait stratumd ready timeout: {}", addr));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn connect_with_retry(addr: SocketAddr, timeout: Duration) -> Result<TcpStream> {
    let start = Instant::now();
    loop {
        match TcpStream::connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(err) if err.kind() == io::ErrorKind::ConnectionRefused => {
                if start.elapsed() >= timeout {
                    return Err(anyhow::anyhow!(
                        "connect timeout after {:?}: {}",
                        timeout,
                        err
                    ));
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(err) => return Err(err.into()),
        }
    }
}

async fn read_json_line(reader: &mut BufReader<OwnedReadHalf>) -> Result<serde_json::Value> {
    let mut line = String::new();
    let read = tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut line))
        .await
        .map_err(|_| anyhow::anyhow!("response timeout"))??;
    if read == 0 {
        return Err(anyhow::anyhow!("connection closed"));
    }
    let value: serde_json::Value = serde_json::from_str(line.trim())?;
    Ok(value)
}

fn extract_job(value: &serde_json::Value) -> Option<StratumJobResponse> {
    if let Some(result) = value.get("result") {
        if result.get("job").is_some() {
            return serde_json::from_value(result.clone()).ok();
        }
    }
    if value.get("method").and_then(|m| m.as_str()) == Some("job") {
        if let Some(params) = value.get("params") {
            if let Ok(job) = serde_json::from_value::<StratumJob>(params.clone()) {
                return Some(StratumJobResponse {
                    login: None,
                    id: job.id.clone(),
                    status: "OK".to_string(),
                    job,
                });
            }
        }
    }
    None
}

fn extract_error_message(value: &serde_json::Value) -> Option<String> {
    value
        .get("error")
        .and_then(|err| err.get("message"))
        .and_then(|msg| msg.as_str())
        .map(|msg| msg.to_string())
}

fn response_id_matches(value: &serde_json::Value, req_id: u64) -> bool {
    value
        .get("id")
        .map(|id| match id {
            serde_json::Value::Number(n) => n.as_u64() == Some(req_id),
            serde_json::Value::String(s) => s.parse::<u64>().ok() == Some(req_id),
            _ => false,
        })
        .unwrap_or(false)
}

async fn wait_for_job(reader: &mut BufReader<OwnedReadHalf>) -> Result<StratumJobResponse> {
    let start = Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(5) {
            return Err(anyhow::anyhow!("job notification timeout"));
        }
        let value = read_json_line(reader).await?;
        if let Some(job) = extract_job(&value) {
            return Ok(job);
        }
    }
}

async fn wait_for_response_id(
    reader: &mut BufReader<OwnedReadHalf>,
    req_id: u64,
) -> Result<serde_json::Value> {
    let start = Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(5) {
            return Err(anyhow::anyhow!(
                "response timeout for request id {}",
                req_id
            ));
        }
        let value = read_json_line(reader).await?;
        if response_id_matches(&value, req_id) {
            return Ok(value);
        }
    }
}

async fn write_json_line(writer: &mut OwnedWriteHalf, value: serde_json::Value) -> Result<()> {
    let payload = format!("{}\n", value);
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

async fn send_submit(
    writer: &mut OwnedWriteHalf,
    id: &str,
    job_id: &str,
    nonce: &str,
    result: &str,
    req_id: u64,
) -> Result<()> {
    let submit_req = json!({
        "id": req_id,
        "jsonrpc": "2.0",
        "method": "submit",
        "params": {
            "id": id,
            "job_id": job_id,
            "nonce": nonce,
            "result": result
        }
    });
    write_json_line(writer, submit_req).await
}

async fn wait_submit_count(mock: &MockMiningRpc, expected: usize, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    loop {
        if mock.submit_calls()?.len() >= expected {
            return Ok(());
        }
        if start.elapsed() >= timeout {
            return Err(anyhow::anyhow!(
                "submit count timeout, expected >= {}, got {}",
                expected,
                mock.submit_calls()?.len()
            ));
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_protocol_login_returns_initial_job() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1, 1, ConsensusStrategy::Dummy))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd = StratumdProcess::spawn(listen, &mock.ws_url(), &[]).await?;

    let stream = connect_with_retry(listen, Duration::from_secs(5)).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login_req = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "login",
        "params": {
            "login": "test",
            "pass": "x",
            "agent": "test-client"
        }
    });
    write_json_line(&mut writer, login_req).await?;

    let job = wait_for_job(&mut reader).await?;
    assert_eq!(job.status, "OK");
    assert!(!job.id.is_empty());
    assert!(!job.job.job_id.is_empty());
    assert!(!job.job.blob.is_empty());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_protocol_submit_returns_ok_on_valid_share() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1, 1, ConsensusStrategy::Dummy))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd = StratumdProcess::spawn(listen, &mock.ws_url(), &[]).await?;

    let stream = connect_with_retry(listen, Duration::from_secs(5)).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login_req = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "login",
        "params": {
            "login": "test",
            "pass": "x",
            "agent": "test-client"
        }
    });
    write_json_line(&mut writer, login_req).await?;
    let job = wait_for_job(&mut reader).await?;

    send_submit(&mut writer, &job.id, &job.job.job_id, "00000000", "00", 2).await?;
    let response = wait_for_response_id(&mut reader, 2).await?;
    assert_eq!(response["result"]["status"].as_str(), Some("OK"));

    wait_submit_count(&mock, 1, Duration::from_secs(2)).await?;
    let calls = mock.submit_calls()?;
    let call = calls
        .first()
        .ok_or_else(|| anyhow::anyhow!("missing submit call"))?;
    assert_eq!(call.nonce, 0);
    assert_eq!(call.extra, job.id);
    assert!(!call.minting_blob.is_empty());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_protocol_keepalive_after_login_returns_status() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1, 1, ConsensusStrategy::Dummy))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd = StratumdProcess::spawn(listen, &mock.ws_url(), &[]).await?;

    let stream = connect_with_retry(listen, Duration::from_secs(5)).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login_req = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "login",
        "params": {
            "login": "test",
            "pass": "x",
            "agent": "test-client"
        }
    });
    write_json_line(&mut writer, login_req).await?;
    let _ = wait_for_job(&mut reader).await?;

    let keep_req = json!({
        "id": 2,
        "jsonrpc": "2.0",
        "method": "keepalived",
        "params": {
            "id": "test"
        }
    });
    write_json_line(&mut writer, keep_req).await?;
    let response = wait_for_response_id(&mut reader, 2).await?;
    assert_eq!(response["result"]["status"].as_str(), Some("KEEPALIVED"));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_security_invalid_nonce_hits_disconnect_threshold() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1, 1, ConsensusStrategy::Dummy))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd =
        StratumdProcess::spawn(listen, &mock.ws_url(), &["--max-invalid-shares", "2"]).await?;

    let stream = connect_with_retry(listen, Duration::from_secs(5)).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login_req = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "login",
        "params": {
            "login": "test",
            "pass": "x",
            "agent": "test-client"
        }
    });
    write_json_line(&mut writer, login_req).await?;
    let job = wait_for_job(&mut reader).await?;

    send_submit(&mut writer, &job.id, &job.job.job_id, "zz", "00", 2).await?;
    let resp1 = wait_for_response_id(&mut reader, 2).await?;
    assert_eq!(
        extract_error_message(&resp1).as_deref(),
        Some("invalid nonce")
    );

    send_submit(&mut writer, &job.id, &job.job.job_id, "zz", "00", 3).await?;
    let resp2 = wait_for_response_id(&mut reader, 3).await?;
    assert_eq!(
        extract_error_message(&resp2).as_deref(),
        Some("invalid nonce")
    );

    let closed = read_json_line(&mut reader).await;
    assert!(
        closed.is_err(),
        "connection should be closed after threshold"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_security_duplicate_share_rejected() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1, 1, ConsensusStrategy::Dummy))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd =
        StratumdProcess::spawn(listen, &mock.ws_url(), &["--max-invalid-shares", "10"]).await?;

    let stream = connect_with_retry(listen, Duration::from_secs(5)).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login_req = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "login",
        "params": {
            "login": "test",
            "pass": "x",
            "agent": "test-client"
        }
    });
    write_json_line(&mut writer, login_req).await?;
    let job = wait_for_job(&mut reader).await?;

    send_submit(&mut writer, &job.id, &job.job.job_id, "00000000", "00", 2).await?;
    let ok = wait_for_response_id(&mut reader, 2).await?;
    assert_eq!(ok["result"]["status"].as_str(), Some("OK"));

    send_submit(&mut writer, &job.id, &job.job.job_id, "00000000", "00", 3).await?;
    let dup = wait_for_response_id(&mut reader, 3).await?;
    assert_eq!(
        extract_error_message(&dup).as_deref(),
        Some("duplicate share")
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_security_job_miss_hits_disconnect_threshold() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1, 1, ConsensusStrategy::Dummy))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd =
        StratumdProcess::spawn(listen, &mock.ws_url(), &["--max-job-misses", "1"]).await?;

    let stream = connect_with_retry(listen, Duration::from_secs(5)).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login_req = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "login",
        "params": {
            "login": "test",
            "pass": "x",
            "agent": "test-client"
        }
    });
    write_json_line(&mut writer, login_req).await?;
    let job = wait_for_job(&mut reader).await?;

    send_submit(
        &mut writer,
        &job.id,
        "0000000000000000",
        "00000000",
        "00",
        2,
    )
    .await?;
    let resp = wait_for_response_id(&mut reader, 2).await?;
    assert_eq!(
        extract_error_message(&resp).as_deref(),
        Some("job not found")
    );

    let closed = read_json_line(&mut reader).await;
    assert!(
        closed.is_err(),
        "connection should be closed after job miss"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_security_stale_share_hits_disconnect_threshold() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1, 1, ConsensusStrategy::Dummy))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd = StratumdProcess::spawn(
        listen,
        &mock.ws_url(),
        &["--max-stale-shares", "1", "--stale-window-secs", "120"],
    )
    .await?;

    let stream = connect_with_retry(listen, Duration::from_secs(5)).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login_req = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "login",
        "params": {
            "login": "test",
            "pass": "x",
            "agent": "test-client"
        }
    });
    write_json_line(&mut writer, login_req).await?;
    let old_job = wait_for_job(&mut reader).await?;

    mock.set_job(build_mint_event(2, 1, ConsensusStrategy::Dummy))?;
    let _new_job = wait_for_job(&mut reader).await?;

    send_submit(
        &mut writer,
        &old_job.id,
        &old_job.job.job_id,
        "00000000",
        "00",
        2,
    )
    .await?;
    let stale = wait_for_response_id(&mut reader, 2).await?;
    assert_eq!(
        extract_error_message(&stale).as_deref(),
        Some("stale share")
    );

    let closed = read_json_line(&mut reader).await;
    assert!(
        closed.is_err(),
        "connection should be closed after stale share"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_security_share_rate_limit_enforced() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1, 1, ConsensusStrategy::Dummy))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd = StratumdProcess::spawn(
        listen,
        &mock.ws_url(),
        &[
            "--share-rate-window-secs",
            "1",
            "--max-shares-per-window",
            "2",
        ],
    )
    .await?;

    let stream = connect_with_retry(listen, Duration::from_secs(5)).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let login_req = json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "login",
        "params": {
            "login": "test",
            "pass": "x",
            "agent": "test-client"
        }
    });
    write_json_line(&mut writer, login_req).await?;
    let job = wait_for_job(&mut reader).await?;

    send_submit(&mut writer, &job.id, &job.job.job_id, "00000000", "00", 2).await?;
    let _ = wait_for_response_id(&mut reader, 2).await?;
    send_submit(&mut writer, &job.id, &job.job.job_id, "00000001", "00", 3).await?;
    let _ = wait_for_response_id(&mut reader, 3).await?;
    send_submit(&mut writer, &job.id, &job.job.job_id, "00000002", "00", 4).await?;
    let resp = wait_for_response_id(&mut reader, 4).await?;
    assert_eq!(
        extract_error_message(&resp).as_deref(),
        Some("rate limited")
    );

    let keep_req = json!({
        "id": 5,
        "jsonrpc": "2.0",
        "method": "keepalived",
        "params": {
            "id": "test"
        }
    });
    write_json_line(&mut writer, keep_req).await?;
    let keep = wait_for_response_id(&mut reader, 5).await?;
    assert_eq!(keep["result"]["status"].as_str(), Some("KEEPALIVED"));

    Ok(())
}
