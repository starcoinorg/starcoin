use anyhow::{Context, Result};
use futures::StreamExt;
use jsonrpc_core::{Error as JsonRpcError, IoHandler, Params};
use jsonrpc_ws_server::{Server as WsServer, ServerBuilder};
use once_cell::sync::Lazy;
use serde_json::json;
use starcoin_config::MinerClientConfig;
use starcoin_miner_client::stratum_client::StratumJobClient;
use starcoin_miner_client::stratum_client_service::{
    StratumClientService, StratumClientServiceServiceFactory,
};
use starcoin_miner_client::stratum_compat::LoginRequest;
use starcoin_miner_client::JobClient;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_time_service::RealTimeService;
use starcoin_types::genesis_config::ConsensusStrategy;
use starcoin_types::system_events::{MintBlockEvent, SealEvent};
use starcoin_types::U256;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Instant;

static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Clone)]
struct MockRpcState {
    current_job: MintBlockEvent,
    submit_count: usize,
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
            submit_count: 0,
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
            let (_minting_blob, _nonce, _extra): (String, u32, String) = params
                .parse()
                .map_err(|_| JsonRpcError::invalid_params("invalid submit params"))?;
            let mut guard = state_for_submit
                .lock()
                .map_err(|_| JsonRpcError::internal_error())?;
            guard.submit_count = guard.submit_count.saturating_add(1);
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

    fn submit_count(&self) -> Result<usize> {
        let guard = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("mock rpc mutex poisoned"))?;
        Ok(guard.submit_count)
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
    async fn spawn(listen: SocketAddr, node_rpc: &str) -> Result<Self> {
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

        let mut child = cmd.spawn().context("spawn stratumd failed")?;
        wait_for_server_ready(&mut child, listen, Duration::from_secs(6)).await?;
        Ok(Self { child })
    }
}

impl Drop for StratumdProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn resolve_stratumd_bin() -> Result<PathBuf> {
    let bin = std::env::var("STRATUMD_BIN").context(
        "STRATUMD_BIN is not set. Point it to standalone starcoin_stratumd binary path.",
    )?;
    let path = PathBuf::from(bin);
    if !path.exists() {
        return Err(anyhow::anyhow!(
            "STRATUMD_BIN path does not exist: {:?}",
            path
        ));
    }
    Ok(path)
}

fn pick_free_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    Ok(listener.local_addr()?.port())
}

fn build_mint_event(number: u64) -> MintBlockEvent {
    let mut minting_blob = vec![0u8; 76];
    minting_blob[0..8].copy_from_slice(&number.to_le_bytes());
    MintBlockEvent::new(
        starcoin_crypto::HashValue::random(),
        ConsensusStrategy::Dummy,
        minting_blob,
        U256::from(1u64),
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

async fn wait_submit_count(mock: &MockMiningRpc, expected: usize, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    loop {
        if mock.submit_count()? >= expected {
            return Ok(());
        }
        if start.elapsed() >= timeout {
            return Err(anyhow::anyhow!(
                "submit count timeout, expected >= {}, got {}",
                expected,
                mock.submit_count()?
            ));
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[stest::test]
#[ignore = "requires external standalone stratumd binary via STRATUMD_BIN"]
async fn test_miner_client_stratum_compat() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;

    let mock = MockMiningRpc::start(build_mint_event(1))?;
    let listen = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), pick_free_port()?);
    let _stratumd = StratumdProcess::spawn(listen, &mock.ws_url()).await?;

    let registry = RegistryService::launch();
    let client_config = MinerClientConfig {
        server: Some(listen.to_string()),
        plugin_path: None,
        miner_thread: 1,
        enable_stderr: false,
    };
    registry.put_shared(client_config).await?;

    let stratum_cli_srv = registry
        .register_by_factory::<StratumClientService, StratumClientServiceServiceFactory>()
        .await?;
    let time_srv = Arc::new(RealTimeService::new());
    let login = LoginRequest {
        login: "test".into(),
        pass: "x".into(),
        agent: "test-client".into(),
        algo: None,
    };
    let job_client = StratumJobClient::new(stratum_cli_srv, time_srv, login);

    let mut jobs = job_client.subscribe().await?;
    let job = tokio::time::timeout(Duration::from_secs(5), jobs.next())
        .await
        .map_err(|_| anyhow::anyhow!("job notification timeout"))?
        .ok_or_else(|| anyhow::anyhow!("job stream closed"))?;
    let extra = job
        .extra
        .clone()
        .ok_or_else(|| anyhow::anyhow!("missing mint extra"))?;

    let seal = SealEvent {
        minting_blob: job.minting_blob.clone(),
        nonce: 0,
        extra: Some(extra),
        hash_result: "00".into(),
    };
    job_client.submit_seal(seal).await?;
    wait_submit_count(&mock, 1, Duration::from_secs(3)).await?;

    let _ = registry.shutdown_system().await;
    Ok(())
}
