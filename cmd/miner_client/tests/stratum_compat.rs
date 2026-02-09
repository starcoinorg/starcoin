use anyhow::Result;
use futures::StreamExt;
use once_cell::sync::Lazy;
use starcoin_config::{BuiltinNetworkID, MinerClientConfig, NodeConfig, StarcoinOpt};
use starcoin_crypto::HashValue;
use starcoin_miner::{DispatchMintBlockTemplate, MinerService};
use starcoin_miner_client::stratum_client::StratumJobClient;
use starcoin_miner_client::stratum_client_service::{
    StratumClientService, StratumClientServiceServiceFactory,
};
use starcoin_miner_client::JobClient;
use starcoin_service_registry::{
    ActorService, EventHandler, RegistryAsyncService, RegistryService, ServiceContext,
    ServiceFactory,
};
use starcoin_stratum::rpc::LoginRequest;
use starcoin_stratum::service::{StratumService, StratumServiceFactory};
use starcoin_stratum::stratum::{Stratum, StratumFactory};
use starcoin_time_service::RealTimeService;
use starcoin_types::block::{BlockBody, BlockTemplate};
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::genesis_config::{ChainId, ConsensusStrategy};
use starcoin_types::system_events::{MinedBlock, SealEvent};
use starcoin_types::U256;
use starcoin_vm2_vm_types::account_address::AccountAddress as Vm2AccountAddress;
use starcoin_vm2_vm_types::on_chain_resource::ChainId as Vm2ChainId;
use std::fs;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration, Instant};

static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn pick_free_port() -> std::io::Result<u16> {
    let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let port = listener.local_addr()?.port();
    Ok(port)
}

fn prepare_config() -> Result<Option<(NodeConfig, SocketAddr)>> {
    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);
    let suffix = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let base_dir = std::env::temp_dir().join(format!(
        "starcoin-miner-client-stratum-test-{}-{}",
        std::process::id(),
        suffix,
    ));
    fs::create_dir_all(&base_dir)?;
    let opt = StarcoinOpt {
        net: Some(BuiltinNetworkID::Dev.into()),
        base_data_dir: Some(base_dir),
        ..StarcoinOpt::default()
    };
    let mut config = NodeConfig::load_with_opt(&opt)?;
    config.stratum.address = Some(Ipv4Addr::LOCALHOST.into());
    let port = match pick_free_port() {
        Ok(port) => port,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("Skipping test: cannot bind local port in this environment.");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    config.stratum.port = Some(port);
    let addr = config.stratum.get_address().expect("stratum address");
    Ok(Some((config, addr)))
}

fn build_block_template(number: u64, timestamp: u64) -> BlockTemplate {
    let parent_hash = HashValue::zero();
    let author = Vm2AccountAddress::from_hex_literal("0x1").expect("valid address");
    let chain_id_v2 = Vm2ChainId::test();
    let metadata = BlockMetadata::new(
        parent_hash,
        timestamp,
        author,
        0,
        number,
        chain_id_v2,
        0,
        vec![],
        0,
    );
    BlockTemplate::new(
        HashValue::zero(),
        HashValue::zero(),
        HashValue::zero(),
        HashValue::zero(),
        HashValue::zero(),
        0,
        BlockBody::new_empty(),
        ChainId::test(),
        U256::from(1u64),
        ConsensusStrategy::Dummy,
        metadata,
        0,
        HashValue::zero(),
        vec![],
    )
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
                sleep(Duration::from_millis(50)).await;
            }
            Err(err) => return Err(err.into()),
        }
    }
}

struct MinedBlockListener {
    tx: mpsc::UnboundedSender<HashValue>,
}

impl ActorService for MinedBlockListener {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<MinedBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MinedBlock>();
        Ok(())
    }
}

impl EventHandler<Self, MinedBlock> for MinedBlockListener {
    fn handle_event(&mut self, msg: MinedBlock, _ctx: &mut ServiceContext<Self>) {
        let MinedBlock(block) = msg;
        let _ = self.tx.send(block.id());
    }
}

struct MinedBlockListenerFactory;

impl ServiceFactory<MinedBlockListener> for MinedBlockListenerFactory {
    fn create(ctx: &mut ServiceContext<MinedBlockListener>) -> Result<MinedBlockListener> {
        let tx = ctx
            .get_shared::<mpsc::UnboundedSender<HashValue>>()?
            .clone();
        Ok(MinedBlockListener { tx })
    }
}

#[stest::test]
async fn test_miner_client_stratum_compat() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((config, addr)) = prepare_config()? else {
        return Ok(());
    };

    let registry = RegistryService::launch();
    registry.put_shared(Arc::new(config)).await?;

    registry.register::<MinerService>().await?;
    registry
        .register_by_factory::<Stratum, StratumFactory>()
        .await?;
    registry
        .register_by_factory::<StratumService, StratumServiceFactory>()
        .await?;

    let miner = registry.service_ref::<MinerService>().await?;
    miner
        .send(DispatchMintBlockTemplate {
            block_template: build_block_template(1, 0),
        })
        .await?;

    let _ = connect_with_retry(addr, Duration::from_secs(5)).await?;

    let (block_tx, mut block_rx) = mpsc::unbounded_channel();
    registry.put_shared(block_tx).await?;
    registry
        .register_by_factory::<MinedBlockListener, MinedBlockListenerFactory>()
        .await?;

    let client_config = MinerClientConfig {
        server: Some(addr.to_string()),
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

    tokio::time::timeout(Duration::from_secs(5), block_rx.recv())
        .await
        .map_err(|_| anyhow::anyhow!("mined block timeout"))?
        .ok_or_else(|| anyhow::anyhow!("mined block channel closed"))?;

    let _ = registry.shutdown_system().await;
    Ok(())
}
