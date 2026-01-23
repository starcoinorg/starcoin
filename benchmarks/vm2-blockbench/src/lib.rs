use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{bail, Result};
use futures::{stream, FutureExt, StreamExt};
use rayon::prelude::*;
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{
    genesis_config::ChainNetwork, BaseConfig, BuiltinNetworkID, DataDirPath, NodeConfig,
    StarcoinOpt, G_DEFAULT_BASE_DATA_DIR,
};
use starcoin_crypto::multi_ed25519::genesis_multi_key_pair as genesis_multi_key_pair_v1;
use starcoin_logger::prelude::*;
use starcoin_miner::{GenerateBlockEvent, MinerService};
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_storage::{Storage, Storage2, Store};
use starcoin_transaction_builder::vm2 as txn_builder2;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_vm2_account_api::{
    message::{AccountRequest, AccountResponse},
    AccountInfo,
};
use starcoin_vm2_account_service::AccountService as AccountService2;
use starcoin_vm2_crypto::multi_ed25519::genesis_multi_key_pair as genesis_multi_key_pair_v2;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::{
    account_address::AccountAddress,
    account_config::association_address,
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use starcoin_vm2_vm_types::state_view::StateReaderExt;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use tokio::time::{sleep, timeout};

mod stats;
pub use stats::{
    recent_block_window_stats, user_tx_since_block, BlockWindowStats, PostPrepareBlockStats,
};

const INIT_BALANCE: u128 = 40_000_000_000;
const TTL_SECS: u64 = 3600;
const CUSTOM_CHAIN_ID: u8 = 121;
const DEFAULT_ACCOUNT_PASSWORD: &str = "";

pub struct DataDir {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TransferStats {
    pub submitted: usize,
    pub executed: usize,
    pub duration_secs: f64,
    pub tps: f64,
    pub block_window: Option<BlockWindowStats>,
    pub post_prepare_blocks: Option<PostPrepareBlockStats>,
}

impl DataDir {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let dir = path.unwrap_or_else(default_bench_data_dir);
        std::fs::create_dir_all(&dir)?;
        Ok(Self { path: dir })
    }
}

fn default_bench_data_dir() -> PathBuf {
    G_DEFAULT_BASE_DATA_DIR.join("vm2-blockbench")
}

pub fn load_configs(data_dir: &DataDir) -> Result<Arc<NodeConfig>> {
    // Build a Proxima-based custom genesis but inject a local association private key so funding works,
    let mut genesis_cfg1 = BuiltinNetworkID::Proxima.genesis_config().clone();
    let (assoc_priv1, assoc_pub1) = genesis_multi_key_pair_v1();
    genesis_cfg1.association_key_pair = (Some(Arc::new(assoc_priv1)), assoc_pub1);
    // Force dummy consensus for fast/CPU-free mining in bench runs.
    genesis_cfg1.consensus_config.strategy = ConsensusStrategy::Dummy.value();
    genesis_cfg1.consensus_config.base_block_time_target = 1;
    // Lift block capacity for stress tests.
    genesis_cfg1.consensus_config.base_block_gas_limit = 20_000_000_000;
    if let starcoin_config::genesis_config::GenesisBlockParameterConfig::Static(ref mut p) =
        genesis_cfg1.genesis_block_parameter
    {
        p.difficulty = 1.into();
    }

    let mut genesis_cfg2 = BuiltinNetworkID::Proxima.genesis_config2().clone();
    genesis_cfg2.stdlib_version = starcoin_types::stdlib::StdlibVersion::Latest;
    let (assoc_priv2, assoc_pub2) = genesis_multi_key_pair_v2();
    genesis_cfg2.association_key_pair = (Some(Arc::new(assoc_priv2)), assoc_pub2);
    // Keep vm2 side consistent with dummy consensus.
    genesis_cfg2.consensus_config.strategy = ConsensusStrategy::Dummy.value();
    genesis_cfg2.consensus_config.base_block_time_target = 1;
    genesis_cfg2.consensus_config.base_block_gas_limit = 20_000_000_000;
    genesis_cfg2.consensus_config.max_transaction_per_block = 10_000;
    if let starcoin_config::genesis_config::vm2::GenesisBlockParameterConfig::Static(ref mut p) =
        genesis_cfg2.genesis_block_parameter
    {
        p.difficulty = 1.into();
    }

    let chain_network = ChainNetwork::new_custom(
        "bench_chain".to_owned(),
        starcoin_types::genesis_config::ChainId::new(CUSTOM_CHAIN_ID),
        genesis_cfg1,
        genesis_cfg2,
    )?;

    let data_dir_path = data_dir.path.join(chain_network.id().chain_name());
    std::fs::create_dir_all(&data_dir_path)?;

    let base_config = BaseConfig {
        net: chain_network,
        base_data_dir: DataDirPath::PathBuf(data_dir.path.clone()),
        data_dir: data_dir_path,
    };

    let mut node_config = NodeConfig::default();
    let opt = StarcoinOpt {
        net: Some(base_config.net().id().clone()),
        base_data_dir: Some(base_config.base_data_dir.as_ref().to_path_buf()),
        ..Default::default()
    };
    node_config.merge_with_opt(&opt, Arc::new(base_config))?;
    // Only mint when there are transactions; avoid empty blocks during bench.
    node_config.miner.disable_mint_empty_block = Some(true);
    node_config.storage.cache_size = Some(20000 * 100);
    Ok(Arc::new(node_config))
}

pub async fn prepare_accounts(
    account_count: u32,
    gas_price: u64,
    max_gas: u64,
    cfg: Arc<NodeConfig>,
    account_svc: ServiceRef<AccountService2>,
    chain_reader: ServiceRef<ChainReaderService>,
    txpool: TxPoolService,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<Vec<AccountInfo>> {
    ensure_association_key(&cfg)?;
    // Always start from the full set of existing accounts.
    let mut accounts = existing_accounts(account_svc.clone()).await?;
    let desired = account_count as usize;
    if accounts.len() >= desired {
        accounts.truncate(desired);
        // Fund all accounts we plan to use (including default/miner) to avoid fee rejections.
        fund_accounts(
            &accounts,
            gas_price,
            max_gas,
            cfg,
            chain_reader,
            txpool,
            storage1,
            storage2,
        )
        .await?;
        return Ok(accounts);
    }

    let need_create = desired - accounts.len();
    let new_accounts = create_accounts(need_create as u32, account_svc.clone()).await?;
    accounts.extend(new_accounts);

    // Fund all accounts we now have (existing + newly created).
    fund_accounts(
        &accounts[..desired],
        gas_price,
        max_gas,
        cfg,
        chain_reader,
        txpool,
        storage1,
        storage2,
    )
    .await?;

    accounts.truncate(desired);
    Ok(accounts)
}

fn ensure_association_key(cfg: &NodeConfig) -> Result<()> {
    let genesis_cfg = cfg.net().genesis_config2();
    if genesis_cfg.association_key_pair.0.is_none() {
        bail!("association private key not available for this network; use a dev/test network");
    }
    Ok(())
}

async fn create_accounts(
    count: u32,
    account_svc: ServiceRef<AccountService2>,
) -> Result<Vec<AccountInfo>> {
    let mut res = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let acc = match account_svc
            .send(AccountRequest::CreateAccount(
                DEFAULT_ACCOUNT_PASSWORD.to_string(),
            ))
            .await??
        {
            AccountResponse::AccountInfo(a) => *a,
            _ => bail!("unexpected account response"),
        };
        res.push(acc);
    }
    Ok(res)
}

async fn existing_accounts(account_svc: ServiceRef<AccountService2>) -> Result<Vec<AccountInfo>> {
    let accounts = match account_svc.send(AccountRequest::GetAccounts()).await?? {
        AccountResponse::AccountList(list) => list
            .into_iter()
            .filter(|a| !a.is_readonly)
            .collect::<Vec<_>>(),
        other => bail!("unexpected account response: {:?}", other),
    };
    Ok(accounts)
}

async fn fund_accounts(
    accounts: &[AccountInfo],
    gas_price: u64,
    max_gas: u64,
    cfg: Arc<NodeConfig>,
    chain_reader: ServiceRef<ChainReaderService>,
    txpool: TxPoolService,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<()> {
    let header = current_header(chain_reader.clone()).await?;
    let expire = cfg.net().time_service().now_secs() + TTL_SECS;
    let genesis_cfg = cfg.net().genesis_config2();

    let multi_state = storage1.get_vm_multi_state(header.id())?;
    let statedb2 = ChainStateDB::new(storage2.clone(), Some(multi_state.state_root2()));
    let mut next_seq = statedb2
        .get_account_resource(association_address())?
        .sequence_number();
    if let Some(pending) = txpool.next_sequence_number2(association_address()) {
        next_seq = next_seq.max(pending);
    }
    let start_seq = next_seq;

    let mut txs: Vec<SignedUserTransaction> = Vec::new();
    for ch in accounts.chunks(funding_batch_size(max_gas)) {
        let addrs: Vec<AccountAddress> = ch.iter().map(|a| a.address).collect();
        let payload = txn_builder2::build_batch_payload_same_amount(addrs, INIT_BALANCE);
        let txn = txn_builder2::create_signed_txn_with_association_account(
            payload,
            next_seq,
            max_gas,
            gas_price,
            expire,
            header.chain_id().id().into(),
            genesis_cfg,
        );
        txs.push(txn);
        next_seq += 1;
    }

    let total_txs = txs.len() as u64;
    txpool.add_txns_multi_signed(
        txs.into_iter()
            .map(MultiSignedUserTransaction::VM2)
            .collect(),
        true,
        None,
    )?;

    let expected_seq = start_seq + total_txs;
    wait_for_association_seq(expected_seq, chain_reader, storage1, storage2).await?;
    Ok(())
}

fn funding_batch_size(max_gas: u64) -> usize {
    const BASE_GAS: u64 = 3_000_000;
    const PER_ACCOUNT: u64 = 2_000_000;
    const MIN_BATCH: usize = 20;
    const MAX_BATCH: usize = 500;
    let usable = max_gas.saturating_sub(BASE_GAS);
    let est = (usable / PER_ACCOUNT).max(1) as usize;
    est.clamp(MIN_BATCH, MAX_BATCH)
}

#[allow(clippy::too_many_arguments)]
pub async fn prepare_transfers(
    accounts: &[AccountInfo],
    amount: u128,
    gas_price: u64,
    max_gas: u64,
    submit_batch: usize,
    total_txns: usize,
    tps_block_window: u64,
    post_prepare_start_block: Option<u64>,
    cfg: Arc<NodeConfig>,
    account_svc: ServiceRef<AccountService2>,
    chain_reader: ServiceRef<ChainReaderService>,
    bus: ServiceRef<BusService>,
    txpool: TxPoolService,
    miner: Option<ServiceRef<MinerService>>,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<TransferStats> {
    let _start = std::time::Instant::now();
    if accounts.len() < 2 {
        return Ok(TransferStats {
            submitted: 0,
            executed: 0,
            duration_secs: 0.0,
            tps: 0.0,
            block_window: None,
            post_prepare_blocks: None,
        });
    }
    let start_block = head_block(chain_reader.clone()).await?;
    let expire = cfg.net().time_service().now_secs() + TTL_SECS;
    let chain_id = start_block.header.chain_id().id().into();
    let multi_state = storage1.get_vm_multi_state(start_block.header.id())?;
    let statedb2 = ChainStateDB::new(storage2.clone(), Some(multi_state.state_root2()));
    let mut head_rx = bus
        .channel::<starcoin_types::system_events::NewHeadBlock>()
        .await?;

    let midpoint = accounts.len() / 2;
    let (senders, receivers) = accounts.split_at(midpoint);
    let sender_count = senders.len();
    if sender_count == 0 || receivers.is_empty() {
        return Ok(TransferStats {
            submitted: 0,
            executed: 0,
            duration_secs: 0.0,
            tps: 0.0,
            block_window: None,
            post_prepare_blocks: None,
        });
    }

    // Unlock senders once to avoid repeated RPCs during signing.
    let unlock_start = std::time::Instant::now();
    let unlock_parallelism = num_cpus::get().clamp(1, 32);
    let unlock_results = stream::iter(senders.iter().map(|sender| {
        let account_svc = account_svc.clone();
        let address = sender.address;
        async move {
            match account_svc
                .send(AccountRequest::UnlockAccount(
                    address,
                    DEFAULT_ACCOUNT_PASSWORD.to_string(),
                    Duration::from_secs(TTL_SECS),
                ))
                .await??
            {
                AccountResponse::AccountInfo(_) => Ok(()),
                other => bail!("unexpected unlock response: {:?}", other),
            }
        }
    }))
    .buffer_unordered(unlock_parallelism)
    .collect::<Vec<_>>()
    .await;
    for res in unlock_results {
        res?;
    }
    let unlock_elapsed = unlock_start.elapsed();
    let unlock_per_ms = if sender_count > 0 {
        unlock_elapsed.as_secs_f64() * 1000.0 / sender_count as f64
    } else {
        0.0
    };
    info!(
        target: "vm2-blockbench",
        "unlock senders done: count={} duration_ms={:.3} per_sender_ms={:.3}",
        sender_count,
        unlock_elapsed.as_secs_f64() * 1000.0,
        unlock_per_ms
    );

    let seq_start = std::time::Instant::now();
    let mut next_seq: Vec<u64> = senders
        .par_iter()
        .map(|sender| {
            let mut seq = statedb2
                .get_account_resource(sender.address)
                .map(|res| res.sequence_number())?;
            if let Some(pending) = txpool.next_sequence_number2(sender.address) {
                seq = seq.max(pending);
            }
            Ok::<_, anyhow::Error>(seq)
        })
        .collect::<Result<Vec<_>>>()?;
    let seq_elapsed = seq_start.elapsed();
    let seq_per_ms = if sender_count > 0 {
        seq_elapsed.as_secs_f64() * 1000.0 / sender_count as f64
    } else {
        0.0
    };
    info!(
        target: "vm2-blockbench",
        "read sender seq done: count={} duration_ms={:.3} per_sender_ms={:.3}",
        sender_count,
        seq_elapsed.as_secs_f64() * 1000.0,
        seq_per_ms
    );

    let total_start = std::time::Instant::now();
    let mut raws: Vec<(usize, RawUserTransaction, AccountAddress)> = Vec::with_capacity(total_txns);
    info!(target: "vm2-blockbench", "build txns start: total={}", total_txns);
    let build_start = std::time::Instant::now();
    for i in 0..total_txns {
        let sender_idx = i % sender_count;
        let sender = &senders[sender_idx];
        let receiver = &receivers[sender_idx];
        let seq = next_seq[sender_idx];
        next_seq[sender_idx] += 1;

        let raw = txn_builder2::build_transfer_txn(
            sender.address,
            receiver.address,
            seq,
            amount,
            gas_price,
            max_gas,
            expire,
            chain_id,
        );
        raws.push((i, raw, sender.address));
    }
    let build_elapsed = build_start.elapsed();
    let build_per_txn_us = if total_txns > 0 {
        build_elapsed.as_secs_f64() * 1_000_000.0 / total_txns as f64
    } else {
        0.0
    };
    info!(
        target: "vm2-blockbench",
        "build txns done: total={} duration_ms={:.3} per_txn_us={:.3}",
        total_txns,
        build_elapsed.as_secs_f64() * 1000.0,
        build_per_txn_us
    );

    let mut txs = Vec::with_capacity(total_txns);
    let sign_parallelism = num_cpus::get().clamp(1, 32);
    let sign_start = std::time::Instant::now();
    info!(target: "vm2-blockbench", "sign txns start: total={}", total_txns);
    let signed = stream::iter(raws.into_iter().map(|(idx, raw, signer)| {
        let account_svc = account_svc.clone();
        async move {
            let sign_call_start = std::time::Instant::now();
            let signed = match account_svc
                .send(AccountRequest::SignTxn {
                    txn: Box::new(raw),
                    signer,
                })
                .await??
            {
                AccountResponse::SignedTxn(t) => *t,
                other => bail!("unexpected sign response: {:?}", other),
            };
            Ok::<_, anyhow::Error>((idx, signed, sign_call_start.elapsed()))
        }
    }))
    .buffer_unordered(sign_parallelism)
    .collect::<Vec<_>>()
    .await;
    let mut sign_call_elapsed = Duration::from_secs(0);
    let mut signed = signed.into_iter().collect::<Result<Vec<_>>>()?;
    signed.sort_by_key(|(idx, _, _)| *idx);
    for (_, signed, elapsed) in signed {
        sign_call_elapsed += elapsed;
        txs.push(MultiSignedUserTransaction::VM2(signed));
    }
    let sign_elapsed = sign_start.elapsed();
    let sign_per_txn_us = if total_txns > 0 {
        sign_elapsed.as_secs_f64() * 1_000_000.0 / total_txns as f64
    } else {
        0.0
    };
    info!(
        target: "vm2-blockbench",
        "sign txns done: total={} duration_ms={:.3} per_txn_us={:.3} sign_call_ms={:.3}",
        total_txns,
        sign_elapsed.as_secs_f64() * 1000.0,
        sign_per_txn_us,
        sign_call_elapsed.as_secs_f64() * 1000.0
    );
    let total_elapsed = total_start.elapsed();
    let total_per_txn_us = if total_txns > 0 {
        total_elapsed.as_secs_f64() * 1_000_000.0 / total_txns as f64
    } else {
        0.0
    };
    info!(
        target: "vm2-blockbench",
        "build+sign txns done: total={} duration_ms={:.3} per_txn_us={:.3}",
        total_txns,
        total_elapsed.as_secs_f64() * 1000.0,
        total_per_txn_us
    );

    // Drain any queued head events before we start counting transfer execution.
    let mut last_head = start_block.header.number();
    while let Some(Some(event)) = head_rx.next().now_or_never() {
        let number = event.executed_block.header().number();
        if number > last_head {
            last_head = number;
        }
    }

    let mut submitted = 0usize;
    let mut executed = 0usize;
    let mut first_submit_wall: Option<Instant> = None;
    let mut first_tx_wall: Option<Instant> = None;
    let mut last_tx_wall: Option<Instant> = None;
    let mut execution_done_wall: Option<Instant> = None;
    let batch = submit_batch.max(1);

    for chunk in txs.chunks(batch) {
        let now = std::time::Instant::now();
        first_submit_wall.get_or_insert(now);
        let res = txpool.add_txns_multi_signed(chunk.to_vec(), true, None)?;
        let accepted = res.iter().filter(|r| r.is_ok()).count();
        submitted += accepted;
        if accepted == 0 {
            let status = txpool.status();
            let first_err = res
                .iter()
                .find_map(|r| r.as_ref().err())
                .map(|e| format!("{}", e))
                .unwrap_or_else(|| "unknown".to_string());
            bail!(
                "no txns accepted in batch (size={}): txpool txn_count={}/{} mem={}/{} senders={} full={} first_err={}",
                chunk.len(),
                status.txn_count,
                status.txn_max_count,
                status.mem,
                status.mem_max,
                status.senders,
                status.is_full,
                first_err,
            );
        }
        if let Some(miner) = miner.as_ref() {
            // Drive block production per batch to avoid txpool propagation delay.
            miner.notify(GenerateBlockEvent::new_break(true))?;
            let prev_head = last_head;
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    bail!("timeout waiting for new head block after batch submit");
                }
                let event = timeout(remaining, head_rx.next())
                    .await
                    .map_err(|_| {
                        anyhow::anyhow!("timeout waiting for new head block after batch submit")
                    })?
                    .ok_or_else(|| anyhow::anyhow!("new head channel closed"))?;
                let before = last_head;
                handle_head_event(
                    &event,
                    &mut last_head,
                    &mut executed,
                    &mut first_tx_wall,
                    &mut last_tx_wall,
                );
                if last_head > before && last_head > prev_head {
                    break;
                }
            }
        }
    }

    // Consume new head events until we see all submitted txns or hit timeout.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if executed >= submitted {
            execution_done_wall.get_or_insert(Instant::now());
            break;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            execution_done_wall.get_or_insert(Instant::now());
            break;
        }
        let event = timeout(remaining, head_rx.next())
            .await
            .ok()
            .and_then(|r| r);
        if let Some(event) = event {
            handle_head_event(
                &event,
                &mut last_head,
                &mut executed,
                &mut first_tx_wall,
                &mut last_tx_wall,
            );
        } else {
            execution_done_wall.get_or_insert(Instant::now());
            break;
        }
    }

    // Measure throughput from the moment we first push txns into the pool until they
    // finish executing (or we give up), so we account for pre-execution pipeline time.
    let duration = {
        let start_wall = first_submit_wall.unwrap_or_else(Instant::now);
        let end_wall = execution_done_wall
            .or(last_tx_wall)
            .unwrap_or_else(Instant::now);
        end_wall
            .saturating_duration_since(start_wall)
            .as_secs_f64()
            .max(1e-6)
    };
    let tps = if duration > 0.0 {
        executed as f64 / duration
    } else {
        0.0
    };
    let block_window = if tps_block_window > 0 {
        recent_block_window_stats(chain_reader.clone(), tps_block_window).await?
    } else {
        None
    };
    let post_prepare_blocks = if let Some(start_block) = post_prepare_start_block {
        Some(user_tx_since_block(start_block, chain_reader.clone()).await?)
    } else {
        None
    };
    Ok(TransferStats {
        submitted,
        executed,
        duration_secs: duration,
        tps,
        block_window,
        post_prepare_blocks,
    })
}

async fn wait_for_association_seq(
    target_seq: u64,
    chain_reader: ServiceRef<ChainReaderService>,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<()> {
    loop {
        let header = current_header(chain_reader.clone()).await?;
        let seq = association_sequence(header.id(), storage1.clone(), storage2.clone())?;
        if seq >= target_seq {
            return Ok(());
        }
        sleep(Duration::from_millis(300)).await;
    }
}

fn association_sequence(
    header_id: starcoin_crypto::HashValue,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<u64> {
    let multi_state = storage1.get_vm_multi_state(header_id)?;
    let db = ChainStateDB::new(storage2, Some(multi_state.state_root2()));
    Ok(db
        .get_account_resource(association_address())?
        .sequence_number())
}

async fn current_header(
    chain_reader: ServiceRef<ChainReaderService>,
) -> Result<starcoin_types::block::BlockHeader> {
    match chain_reader.send(ChainRequest::CurrentHeader()).await?? {
        ChainResponse::BlockHeader(h) => Ok(*h),
        _ => bail!("unexpected header resp"),
    }
}

pub async fn head_block(
    chain_reader: ServiceRef<ChainReaderService>,
) -> Result<starcoin_types::block::Block> {
    match chain_reader.send(ChainRequest::HeadBlock()).await?? {
        ChainResponse::Block(b) => Ok(*b),
        _ => bail!("unexpected head block response"),
    }
}

fn handle_head_event(
    event: &starcoin_types::system_events::NewHeadBlock,
    last_head: &mut u64,
    executed: &mut usize,
    first_tx_wall: &mut Option<Instant>,
    last_tx_wall: &mut Option<Instant>,
) {
    let number = event.executed_block.header().number();
    if number <= *last_head {
        return;
    }
    *last_head = number;
    let block = event.executed_block.block();
    let tx_count = block.body.transactions.len() + block.body.transactions2.len();
    if tx_count > 0 {
        let now = Instant::now();
        if first_tx_wall.is_none() {
            *first_tx_wall = Some(now);
        }
        *last_tx_wall = Some(now);
        *executed += tx_count;
    }
}
