use anyhow::Result;
use once_cell::sync::Lazy;
use serde_json::json;
use starcoin_config::{BuiltinNetworkID, NodeConfig, StarcoinOpt};
use starcoin_crypto::HashValue;
use starcoin_miner::{DispatchMintBlockTemplate, MinerService};
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceRef};
use starcoin_stratum::diff_manager::DifficultyManager;
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
use std::time::Duration as StdDuration;
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

fn build_block_template_with(
    number: u64,
    timestamp: u64,
    difficulty: U256,
    strategy: ConsensusStrategy,
) -> BlockTemplate {
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
        difficulty,
        strategy,
        metadata,
        0,
        HashValue::zero(),
        vec![],
    )
}

fn build_block_template(number: u64, timestamp: u64) -> BlockTemplate {
    build_block_template_with(
        number,
        timestamp,
        U256::from(1u64),
        ConsensusStrategy::Dummy,
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

async fn wait_for_error_message(reader: &mut BufReader<OwnedReadHalf>) -> Result<String> {
    let start = Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(5) {
            return Err(anyhow::anyhow!("response timeout"));
        }
        let value = read_json_line(reader).await?;
        if let Some(msg) = extract_error_message(&value) {
            return Ok(msg);
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

async fn try_wait_for_job(
    reader: &mut BufReader<OwnedReadHalf>,
    timeout_dur: Duration,
) -> Result<Option<StratumJobResponse>> {
    match tokio::time::timeout(timeout_dur, async {
        loop {
            let value = read_json_line(reader).await?;
            if let Some(job) = extract_job(&value) {
                return Ok(job);
            }
        }
    })
    .await
    {
        Ok(job) => job.map(Some),
        Err(_) => Ok(None),
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

async fn start_registry(
    config: NodeConfig,
) -> Result<(ServiceRef<RegistryService>, ServiceRef<MinerService>)> {
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
    Ok((registry, miner))
}

async fn dispatch_template(
    miner: &starcoin_service_registry::ServiceRef<MinerService>,
    number: u64,
) -> Result<()> {
    miner
        .send(DispatchMintBlockTemplate {
            block_template: build_block_template(number, number),
        })
        .await?;
    Ok(())
}

async fn dispatch_template_with(
    miner: &starcoin_service_registry::ServiceRef<MinerService>,
    number: u64,
    difficulty: U256,
    strategy: ConsensusStrategy,
) -> Result<()> {
    miner
        .send(DispatchMintBlockTemplate {
            block_template: build_block_template_with(number, number, difficulty, strategy),
        })
        .await?;
    Ok(())
}

#[stest::test]
async fn test_protocol_login_returns_initial_job() -> Result<()> {
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
async fn test_protocol_submit_returns_ok_on_valid_share() -> Result<()> {
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
async fn test_protocol_keepalived_returns_status() -> Result<()> {
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

#[stest::test]
async fn test_security_invalid_nonce_hits_disconnect_threshold() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((mut config, addr)) = prepare_config()? else {
        return Ok(());
    };
    config.stratum.max_invalid_shares = Some(2);

    let (registry, miner) = start_registry(config).await?;
    dispatch_template(&miner, 1).await?;
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

    send_submit(&mut writer, &job.id, &job.job.job_id, "zz", "00", 2).await?;
    let resp1 = read_json_line(&mut reader).await?;
    assert_eq!(
        extract_error_message(&resp1).as_deref(),
        Some("invalid nonce")
    );

    send_submit(&mut writer, &job.id, &job.job.job_id, "zz", "00", 3).await?;
    let resp2 = read_json_line(&mut reader).await?;
    assert_eq!(
        extract_error_message(&resp2).as_deref(),
        Some("invalid nonce")
    );

    let closed = read_json_line(&mut reader).await;
    assert!(
        closed.is_err(),
        "connection should be closed after threshold"
    );

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[stest::test]
async fn test_security_duplicate_share_rejected() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((mut config, addr)) = prepare_config()? else {
        return Ok(());
    };
    config.stratum.max_invalid_shares = Some(10);

    let (registry, miner) = start_registry(config).await?;
    dispatch_template(&miner, 1).await?;
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

    send_submit(&mut writer, &job.id, &job.job.job_id, "00000000", "00", 2).await?;
    let _ = read_json_line(&mut reader).await?;

    send_submit(&mut writer, &job.id, &job.job.job_id, "00000000", "00", 3).await?;
    let resp2 = read_json_line(&mut reader).await?;
    assert_eq!(
        extract_error_message(&resp2).as_deref(),
        Some("duplicate share")
    );

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[stest::test]
async fn test_security_job_miss_hits_disconnect_threshold() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((mut config, addr)) = prepare_config()? else {
        return Ok(());
    };
    config.stratum.max_job_misses = Some(1);

    let (registry, miner) = start_registry(config).await?;
    dispatch_template(&miner, 1).await?;
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

    send_submit(
        &mut writer,
        &job.id,
        "0000000000000000",
        "00000000",
        "00",
        2,
    )
    .await?;
    let resp = read_json_line(&mut reader).await?;
    assert_eq!(
        extract_error_message(&resp).as_deref(),
        Some("job not found")
    );

    let closed = read_json_line(&mut reader).await;
    assert!(
        closed.is_err(),
        "connection should be closed after job miss"
    );

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[stest::test]
async fn test_security_stale_share_hits_disconnect_threshold() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((mut config, addr)) = prepare_config()? else {
        return Ok(());
    };
    config.stratum.max_stale_shares = Some(1);
    config.stratum.stale_window_secs = Some(60);

    let (registry, miner) = start_registry(config).await?;
    dispatch_template(&miner, 1).await?;
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

    dispatch_template(&miner, 2).await?;
    sleep(Duration::from_millis(100)).await;

    send_submit(&mut writer, &job.id, &job.job.job_id, "00000000", "00", 2).await?;
    let msg = wait_for_error_message(&mut reader).await?;
    assert_eq!(msg, "stale share");

    let closed = read_json_line(&mut reader).await;
    assert!(
        closed.is_err(),
        "connection should be closed after stale share"
    );

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[stest::test]
async fn test_security_share_rate_limit_enforced() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((mut config, addr)) = prepare_config()? else {
        return Ok(());
    };
    config.stratum.share_rate_window_secs = Some(1);
    config.stratum.max_shares_per_window = Some(2);

    let (registry, miner) = start_registry(config).await?;
    dispatch_template(&miner, 1).await?;
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

    send_submit(&mut writer, &job.id, &job.job.job_id, "00000000", "00", 2).await?;
    let _ = read_json_line(&mut reader).await?;
    send_submit(&mut writer, &job.id, &job.job.job_id, "00000001", "00", 3).await?;
    let _ = read_json_line(&mut reader).await?;
    send_submit(&mut writer, &job.id, &job.job.job_id, "00000002", "00", 4).await?;
    let resp = read_json_line(&mut reader).await?;
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
    let keep = read_json_line(&mut reader).await?;
    assert_eq!(keep["result"]["status"].as_str(), Some("KEEPALIVED"));

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[stest::test]
async fn test_security_low_difficulty_share_rejected() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((mut config, addr)) = prepare_config()? else {
        return Ok(());
    };
    config.stratum.max_invalid_shares = Some(200);

    let (registry, miner) = start_registry(config).await?;
    dispatch_template_with(
        &miner,
        1,
        U256::from(10_000_000u64),
        ConsensusStrategy::Keccak,
    )
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

    let mut rejected = false;
    for i in 0..20u32 {
        send_submit(
            &mut writer,
            &job.id,
            &job.job.job_id,
            &format!("{:08x}", i),
            "00",
            (i + 2) as u64,
        )
        .await?;
        let resp = read_json_line(&mut reader).await?;
        if extract_error_message(&resp).as_deref() == Some("low difficulty share") {
            rejected = true;
            break;
        }
    }
    assert!(
        rejected,
        "expected at least one low difficulty rejection in bounded attempts"
    );

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[stest::test]
async fn test_security_protocol_invalid_params_threshold_disconnects() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((config, addr)) = prepare_config()? else {
        return Ok(());
    };

    let (registry, miner) = start_registry(config).await?;
    dispatch_template(&miner, 1).await?;
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
    let _ = wait_for_job(&mut reader).await?;

    for i in 0..59u64 {
        let bad_submit = json!({
            "id": i + 2,
            "jsonrpc": "2.0",
            "method": "submit",
            "params": {}
        });
        write_json_line(&mut writer, bad_submit).await?;
        let resp = read_json_line(&mut reader).await?;
        assert!(
            extract_error_message(&resp).is_some(),
            "expected protocol error response before threshold"
        );
    }
    let keep_req = json!({
        "id": 1000,
        "jsonrpc": "2.0",
        "method": "keepalived",
        "params": {
            "id": "test"
        }
    });
    write_json_line(&mut writer, keep_req).await?;
    let keep = read_json_line(&mut reader).await?;
    assert_eq!(keep["result"]["status"].as_str(), Some("KEEPALIVED"));
    let threshold_submit = json!({
        "id": 1001,
        "jsonrpc": "2.0",
        "method": "submit",
        "params": {}
    });
    write_json_line(&mut writer, threshold_submit).await?;
    let _ = read_json_line(&mut reader).await;

    let closed = read_json_line(&mut reader).await;
    assert!(
        closed.is_err(),
        "connection should be closed after protocol invalid-param threshold"
    );

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[stest::test]
async fn test_security_protocol_worker_mismatch_threshold_disconnects() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((config, addr)) = prepare_config()? else {
        return Ok(());
    };

    let (registry, miner) = start_registry(config).await?;
    dispatch_template(&miner, 1).await?;
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
    let wrong_id = if job.id == "00000000" {
        "11111111"
    } else {
        "00000000"
    };

    for i in 0..59u64 {
        send_submit(
            &mut writer,
            wrong_id,
            &job.job.job_id,
            &format!("{:08x}", i),
            "00",
            i + 2,
        )
        .await?;
        let resp = read_json_line(&mut reader).await?;
        assert_eq!(
            extract_error_message(&resp).as_deref(),
            Some("worker mismatch")
        );
    }
    let keep_req = json!({
        "id": 1000,
        "jsonrpc": "2.0",
        "method": "keepalived",
        "params": {
            "id": "test"
        }
    });
    write_json_line(&mut writer, keep_req).await?;
    let keep = read_json_line(&mut reader).await?;
    assert_eq!(keep["result"]["status"].as_str(), Some("KEEPALIVED"));
    send_submit(
        &mut writer,
        wrong_id,
        &job.job.job_id,
        "00ffff00",
        "00",
        1001,
    )
    .await?;
    let _ = read_json_line(&mut reader).await;

    let closed = read_json_line(&mut reader).await;
    assert!(
        closed.is_err(),
        "connection should be closed after protocol worker-mismatch threshold"
    );

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[stest::test]
async fn test_regression_connection_survives_job_churn_and_mixed_submits() -> Result<()> {
    let _guard = TEST_MUTEX.lock().await;
    let Some((mut config, addr)) = prepare_config()? else {
        return Ok(());
    };
    config.stratum.max_invalid_shares = Some(200);
    config.stratum.max_job_misses = Some(200);
    config.stratum.max_stale_shares = Some(200);
    config.stratum.stale_window_secs = Some(120);

    let (registry, miner) = start_registry(config).await?;
    dispatch_template(&miner, 1).await?;
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
    let mut previous_job = wait_for_job(&mut reader).await?;

    let mut req_id = 2u64;
    let mut saw_job_change = false;
    let mut stale_like_rejects = 0u32;
    for round in 0..30u64 {
        dispatch_template(&miner, round + 2).await?;
        let latest_job = match try_wait_for_job(&mut reader, Duration::from_millis(800)).await? {
            Some(job) => job,
            None => previous_job.clone(),
        };
        let changed = latest_job.job.job_id != previous_job.job.job_id;
        if changed {
            saw_job_change = true;
        }

        // A real miner can still submit shares from the previous job during job churn.
        send_submit(
            &mut writer,
            &previous_job.id,
            &previous_job.job.job_id,
            &format!("{:08x}", round),
            "00",
            req_id,
        )
        .await?;
        let stale_resp = wait_for_response_id(&mut reader, req_id).await?;
        req_id = req_id.saturating_add(1);
        let stale_msg = extract_error_message(&stale_resp);
        if changed {
            assert!(
                matches!(
                    stale_msg.as_deref(),
                    Some("stale share") | Some("job not found")
                ),
                "expected stale-style reject for previous job, got: {}",
                stale_resp
            );
            stale_like_rejects = stale_like_rejects.saturating_add(1);
        } else {
            let is_ok = stale_resp["result"]["status"].as_str() == Some("OK");
            let is_stale_like = matches!(
                stale_msg.as_deref(),
                Some("stale share") | Some("job not found")
            );
            assert!(
                is_ok || is_stale_like,
                "unexpected response for previous-job submit: {}",
                stale_resp
            );
            if is_stale_like {
                stale_like_rejects = stale_like_rejects.saturating_add(1);
            }
        }

        // Latest job submit should still be accepted on the same long-lived connection.
        send_submit(
            &mut writer,
            &latest_job.id,
            &latest_job.job.job_id,
            &format!("{:08x}", 0x8000_0000u64.saturating_add(round)),
            "00",
            req_id,
        )
        .await?;
        let ok_resp = wait_for_response_id(&mut reader, req_id).await?;
        req_id = req_id.saturating_add(1);
        let is_ok = ok_resp["result"]["status"].as_str() == Some("OK");
        let msg = extract_error_message(&ok_resp);
        let is_stale_like = matches!(msg.as_deref(), Some("stale share") | Some("job not found"));
        assert!(
            is_ok || is_stale_like,
            "unexpected submit result for latest job: {}",
            ok_resp
        );

        let keep_req = json!({
            "id": req_id,
            "jsonrpc": "2.0",
            "method": "keepalived",
            "params": {
                "id": "test"
            }
        });
        write_json_line(&mut writer, keep_req).await?;
        let keep_resp = wait_for_response_id(&mut reader, req_id).await?;
        req_id = req_id.saturating_add(1);
        assert_eq!(keep_resp["result"]["status"].as_str(), Some("KEEPALIVED"));

        previous_job = latest_job;
    }

    let final_keep_req = json!({
        "id": req_id,
        "jsonrpc": "2.0",
        "method": "keepalived",
        "params": {
            "id": "test"
        }
    });
    write_json_line(&mut writer, final_keep_req).await?;
    let final_keep = wait_for_response_id(&mut reader, req_id).await?;
    assert_eq!(final_keep["result"]["status"].as_str(), Some("KEEPALIVED"));
    assert!(
        saw_job_change || stale_like_rejects > 0,
        "expected observed churn via job transition or stale-style rejects"
    );
    assert!(
        stale_like_rejects > 0,
        "expected stale/job-miss rejects under churn"
    );

    let _ = registry.shutdown_system().await;
    Ok(())
}

#[test]
fn test_vardiff_increase_decrease_decay() {
    let mut dm = DifficultyManager::new();
    dm.difficulty = U256::from(2000u64);
    dm.avg_share_time = 2.0;
    dm.submits_since_last_update = 9;
    dm.last_update = std::time::Instant::now() - StdDuration::from_secs(35);
    assert!(dm.try_update("test".to_string()));
    assert_eq!(dm.difficulty, U256::from(4000u64));

    dm.difficulty = U256::from(10000u64);
    dm.avg_share_time = 10.0;
    dm.submits_since_last_update = 2;
    dm.last_update = std::time::Instant::now() - StdDuration::from_secs(100);
    assert!(dm.try_update("test".to_string()));
    assert_eq!(dm.difficulty, U256::from(5882u64));

    dm.difficulty = U256::from(8000u64);
    dm.last_share = std::time::Instant::now() - StdDuration::from_secs(40);
    dm.last_decay = std::time::Instant::now() - StdDuration::from_secs(40);
    assert!(dm.maybe_decay("test"));
    assert_eq!(dm.difficulty, U256::from(4000u64));
}

#[test]
fn test_vardiff_skip_update_when_min_period_not_met() {
    let mut dm = DifficultyManager::new();
    dm.difficulty = U256::from(2000u64);
    dm.submits_since_last_update = 20;
    dm.last_update = std::time::Instant::now() - StdDuration::from_secs(5);
    let prev_diff = dm.difficulty;
    assert!(!dm.try_update("test".to_string()));
    assert_eq!(dm.difficulty, prev_diff);
}

#[test]
fn test_vardiff_skip_update_when_insufficient_samples() {
    let mut dm = DifficultyManager::new();
    dm.difficulty = U256::from(2000u64);
    dm.submits_since_last_update = 0;
    dm.last_update = std::time::Instant::now() - StdDuration::from_secs(40);
    let prev_diff = dm.difficulty;
    assert!(!dm.try_update("test".to_string()));
    assert_eq!(dm.difficulty, prev_diff);
}

#[test]
fn test_vardiff_stays_when_avg_time_in_drift_band() {
    let mut dm = DifficultyManager::new();
    dm.difficulty = U256::from(4000u64);
    dm.avg_share_time = 10.0;
    dm.submits_since_last_update = 3;
    dm.last_update = std::time::Instant::now() - StdDuration::from_secs(40);
    assert!(!dm.try_update("test".to_string()));
    assert_eq!(dm.difficulty, U256::from(4000u64));
}

#[test]
fn test_vardiff_decay_stops_at_minimum() {
    let mut dm = DifficultyManager::new();
    dm.difficulty = U256::from(2000u64);
    dm.last_share = std::time::Instant::now() - StdDuration::from_secs(40);
    dm.last_decay = std::time::Instant::now() - StdDuration::from_secs(40);
    assert!(!dm.maybe_decay("test"));
    assert_eq!(dm.difficulty, U256::from(2000u64));
}
