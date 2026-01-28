use anyhow::Result;
use once_cell::sync::Lazy;
use serde_json::json;
use starcoin_config::{BuiltinNetworkID, NodeConfig, StarcoinOpt};
use starcoin_crypto::HashValue;
use starcoin_miner::{DispatchMintBlockTemplate, MinerService};
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_stratum::rpc::StratumJobResponse;
use starcoin_stratum::service::{StratumService, StratumServiceFactory};
use starcoin_stratum::stratum::{Stratum, StratumFactory};
use starcoin_types::block::{BlockBody, BlockTemplate};
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::genesis_config::{ChainId, ConsensusStrategy};
use starcoin_types::U256;
use starcoin_vm2_vm_types::account_address::AccountAddress as Vm2AccountAddress;
use starcoin_vm2_vm_types::on_chain_resource::ChainId as Vm2ChainId;
use std::fs;
use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
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
        "starcoin-stratum-test-{}-{}",
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
        ConsensusStrategy::CryptoNight,
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
            if let Some(result) = params.get("result") {
                if let Ok(job) = serde_json::from_value(result.clone()) {
                    return Some(job);
                }
            }
            if let Ok(job) = serde_json::from_value(params.clone()) {
                return Some(job);
            }
        }
    }
    None
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

async fn write_json_line(writer: &mut OwnedWriteHalf, value: serde_json::Value) -> Result<()> {
    let payload = format!("{}\n", value);
    writer.write_all(payload.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

#[stest::test]
async fn test_login_request() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((config, addr)) = prepare_config()? else {
        return Ok(());
    };

    let registry = RegistryService::launch();
    registry.put_shared(Arc::new(config)).await?;

    let result = tokio::time::timeout(Duration::from_secs(20), async {
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

        sleep(Duration::from_millis(100)).await;

        let stream = connect_with_retry(addr, Duration::from_secs(5)).await?;
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

        Ok::<(), anyhow::Error>(())
    })
    .await
    .map_err(|_| anyhow::anyhow!("test timeout"))?;

    let _ = registry.shutdown_system().await;
    result
}

#[stest::test]
async fn test_submit_request() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((config, addr)) = prepare_config()? else {
        return Ok(());
    };

    let registry = RegistryService::launch();
    registry.put_shared(Arc::new(config)).await?;

    let result = tokio::time::timeout(Duration::from_secs(20), async {
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

        sleep(Duration::from_millis(100)).await;

        let stream = connect_with_retry(addr, Duration::from_secs(5)).await?;
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

        let submit_req = json!({
            "id": 2,
            "jsonrpc": "2.0",
            "method": "submit",
            "params": {
                "id": job.id,
                "job_id": job.job.job_id,
                "nonce": "00000000",
                "result": "00"
            }
        });
        write_json_line(&mut writer, submit_req).await?;

        let response = read_json_line(&mut reader).await?;
        let status = response["result"]["status"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing submit status"))?;
        assert_eq!(status, "OK");

        Ok::<(), anyhow::Error>(())
    })
    .await
    .map_err(|_| anyhow::anyhow!("test timeout"))?;

    let _ = registry.shutdown_system().await;
    result
}

#[stest::test]
async fn test_keepalived_request() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((config, addr)) = prepare_config()? else {
        return Ok(());
    };

    let registry = RegistryService::launch();
    registry.put_shared(Arc::new(config)).await?;

    let result = tokio::time::timeout(Duration::from_secs(20), async {
        registry.register::<MinerService>().await?;
        registry
            .register_by_factory::<Stratum, StratumFactory>()
            .await?;
        registry
            .register_by_factory::<StratumService, StratumServiceFactory>()
            .await?;

        let stream = connect_with_retry(addr, Duration::from_secs(5)).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let keep_req = json!({
            "id": 2,
            "jsonrpc": "2.0",
            "method": "keepalived",
            "params": {
                "id": "test"
            }
        });
        write_json_line(&mut writer, keep_req).await?;

        let response = read_json_line(&mut reader).await?;
        let status = response["result"]["status"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing keepalived status"))?;
        assert_eq!(status, "KEEPALIVED");

        Ok::<(), anyhow::Error>(())
    })
    .await
    .map_err(|_| anyhow::anyhow!("test timeout"))?;

    let _ = registry.shutdown_system().await;
    result
}
