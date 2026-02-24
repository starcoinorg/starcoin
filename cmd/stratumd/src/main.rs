use anyhow::Result;
use byteorder::{BigEndian, ByteOrder, LittleEndian, WriteBytesExt};
use clap::Parser;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_config::{Connect, StratumLimits, StratumPplnsConfig};
use starcoin_consensus::{difficult_to_target, Consensus};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::{AsyncRpcClient, ConnSource};
use starcoin_stratumd::codec::{JsonStreamCodec, Separator};
use starcoin_stratumd::pplns_store::{
    build_pplns_store, CandidateRecord, CandidateStatus, PendingSubmitRecord, PplnsStore,
    ShareRecord,
};
use starcoin_stratumd::rpc::{
    JobId, LoginRequest, MinerWorker, ShareRequest, Status, StratumJobResponse,
    SubmitShareResponse, WorkerId,
};
use starcoin_stratumd::target_hex_to_difficulty;
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::U256;
use starcoin_vm2_vm_types::account_config::events::BlockRewardEvent as BlockRewardEventV2;
use starcoin_vm_types::account_config::events::BlockRewardEvent as BlockRewardEventV1;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};
use tokio_util::codec::Framed;

const OUTBOUND_CHANNEL_CAP: usize = 128;
const MAX_LOGIN_LEN: usize = 256;
const MAX_PASS_LEN: usize = 256;
const MAX_AGENT_LEN: usize = 128;
const READ_IDLE_TIMEOUT_SECS: u64 = 600;
const WRITE_DRAIN_TIMEOUT_SECS: u64 = 2;
const REQ_WINDOW_SECS: u64 = 10;
const MAX_REQS_PER_WINDOW: u32 = 100;
const PROTOCOL_ERROR_WINDOW_SECS: u64 = 120;
const MAX_PROTOCOL_ERRORS: u32 = 60;

const ERROR_WINDOW_SECS: u64 = 120;
const MAX_RECENT_JOBS: usize = 512;
const STATS_LOG_INTERVAL_SECS: u64 = 60;
const MAX_NONCE_HEX_LEN: usize = 8;
const JOB_ID_HEX_LEN: usize = 16;
const WORKER_ID_HEX_LEN: usize = 8;
const TARGET_HEX_LEN: usize = 16;
const SHARE_FLUSH_BATCH_SIZE: u64 = 64;

#[derive(Parser, Debug, Clone)]
#[command(name = "starcoin_stratumd")]
#[command(about = "Standalone Stratum gateway process")]
struct Opt {
    #[arg(long, default_value = "0.0.0.0:9888")]
    listen: SocketAddr,

    #[arg(long, default_value = "ws://127.0.0.1:9870")]
    node_rpc: String,

    #[arg(long, default_value_t = 500)]
    job_poll_ms: u64,

    #[arg(long, default_value_t = 600)]
    share_dedup_window_secs: u64,

    #[arg(long, default_value_t = 120)]
    stale_window_secs: u64,

    #[arg(long, default_value_t = 10)]
    share_rate_window_secs: u64,

    #[arg(long, default_value_t = 200)]
    max_shares_per_window: u32,

    #[arg(long, default_value_t = 60)]
    max_invalid_shares: u32,

    #[arg(long, default_value_t = 60)]
    max_job_misses: u32,

    #[arg(long, default_value_t = 60)]
    max_stale_shares: u32,

    #[arg(long, default_value_t = 1024)]
    max_workers_per_account: usize,

    #[arg(long, default_value_t = false)]
    pplns_enabled: bool,

    #[arg(long, default_value_t = 20_000)]
    pplns_window_shares: u64,

    #[arg(long, default_value_t = 6)]
    pplns_confirmations: u64,

    #[arg(long, default_value_t = 10)]
    pplns_settlement_interval_secs: u64,

    #[arg(long, default_value_t = 3_600)]
    pplns_batch_period_secs: u64,

    #[arg(long, default_value_t = 160_000)]
    pplns_max_retained_shares: u64,

    #[arg(long, default_value_t = 4_096)]
    pplns_max_retained_candidates: usize,

    #[arg(long)]
    pplns_database_url: Option<String>,
}

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

#[derive(Debug, Clone)]
struct ErrorCounter {
    first: Instant,
    count: u32,
}

#[derive(Debug, Clone)]
struct RateCounter {
    first: Instant,
    count: u32,
}

#[derive(Debug, Clone)]
struct JobCacheEntry {
    ts: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ShareKey {
    job_id: [u8; 8],
    nonce: u32,
    extra: [u8; 4],
}

#[derive(Debug, Default, Clone)]
struct ShareStats {
    accepted: u64,
    invalid: u64,
    duplicate: u64,
    stale: u64,
    job_miss: u64,
    rate_limited: u64,
}

struct WorkerSession {
    worker: MinerWorker,
    out_tx: mpsc::Sender<String>,
}

struct GatewayState {
    uid: u32,
    limits: StratumLimits,
    workers: HashMap<WorkerId, WorkerSession>,
    share_dedup: HashMap<WorkerId, HashMap<ShareKey, Instant>>,
    invalid_share_counters: HashMap<WorkerId, ErrorCounter>,
    job_miss_counters: HashMap<WorkerId, ErrorCounter>,
    stale_share_counters: HashMap<WorkerId, ErrorCounter>,
    share_rate: HashMap<WorkerId, RateCounter>,
    recent_jobs: HashMap<[u8; 8], JobCacheEntry>,
    recent_job_queue: VecDeque<[u8; 8]>,
    solved_jobs: HashMap<[u8; 8], Instant>,
    current_job: Option<MintBlockEvent>,
    account_workers: HashMap<String, HashSet<WorkerId>>,
    worker_accounts: HashMap<WorkerId, String>,
    stats: ShareStats,
    last_stats_log: Instant,
    forwarded_blocks: u64,
    accepted_share_seq: u64,
}

impl GatewayState {
    fn new(limits: StratumLimits) -> Self {
        Self {
            uid: 1,
            limits,
            workers: HashMap::new(),
            share_dedup: HashMap::new(),
            invalid_share_counters: HashMap::new(),
            job_miss_counters: HashMap::new(),
            stale_share_counters: HashMap::new(),
            share_rate: HashMap::new(),
            recent_jobs: HashMap::new(),
            recent_job_queue: VecDeque::new(),
            solved_jobs: HashMap::new(),
            current_job: None,
            account_workers: HashMap::new(),
            worker_accounts: HashMap::new(),
            stats: ShareStats::default(),
            last_stats_log: Instant::now(),
            forwarded_blocks: 0,
            accepted_share_seq: 0,
        }
    }

    fn next_id(&mut self) -> u32 {
        let next = self.uid;
        self.uid = self.uid.saturating_add(1);
        next
    }

    fn next_share_seq(&mut self) -> u64 {
        self.accepted_share_seq = self.accepted_share_seq.saturating_add(1);
        self.accepted_share_seq
    }

    fn account_from_login(login: &str) -> String {
        login.split('.').next().unwrap_or(login).to_string()
    }

    fn drop_worker_state(&mut self, worker_id: WorkerId) {
        self.workers.remove(&worker_id);
        self.share_dedup.remove(&worker_id);
        self.invalid_share_counters.remove(&worker_id);
        self.job_miss_counters.remove(&worker_id);
        self.stale_share_counters.remove(&worker_id);
        self.share_rate.remove(&worker_id);
        if let Some(account) = self.worker_accounts.remove(&worker_id) {
            if let Some(set) = self.account_workers.get_mut(&account) {
                set.remove(&worker_id);
                if set.is_empty() {
                    self.account_workers.remove(&account);
                }
            }
        }
    }

    fn register_worker(
        &mut self,
        login: LoginRequest,
        out_tx: mpsc::Sender<String>,
    ) -> Result<(WorkerId, StratumJobResponse)> {
        let account = Self::account_from_login(&login.login);
        let account_worker_len = self.account_workers.get(&account).map_or(0, HashSet::len);
        if account_worker_len >= self.limits.max_workers_per_account {
            return Err(anyhow::anyhow!("too many workers for account"));
        }

        let sub_id = self.next_id();
        let worker = MinerWorker::new(sub_id, login);
        let worker_id = worker.worker_id;

        let current_job = self
            .current_job
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no job"))?
            .clone();

        let job = Self::get_downstream_job(&worker, true, &current_job);
        self.workers
            .insert(worker_id, WorkerSession { worker, out_tx });
        self.account_workers
            .entry(account.clone())
            .or_default()
            .insert(worker_id);
        self.worker_accounts.insert(worker_id, account);
        Ok((worker_id, job))
    }

    fn get_downstream_job(
        miner: &MinerWorker,
        set_login: bool,
        upstream_event: &MintBlockEvent,
    ) -> StratumJobResponse {
        let login = miner.base_info.clone();
        let desired_diff = miner.diff_manager.read().unwrap().difficulty;
        let network_diff = upstream_event.difficulty;
        let target = starcoin_stratumd::difficulty_to_target_hex(desired_diff);
        if target.len() != TARGET_HEX_LEN || !Self::is_hex(&target) {
            warn!(
                target: "stratum_server",
                "unexpected target hex len:{} target:{}",
                target.len(),
                target
            );
        }
        info!(
            target: "stratum_server",
            "set downstream job diff:{:?} (worker:{:?}, network:{:?})",
            target_hex_to_difficulty(&target).unwrap_or_else(|_| U256::from(1u64)),
            desired_diff,
            network_diff
        );

        StratumJobResponse::from(
            upstream_event,
            if set_login { Some(login) } else { None },
            miner.worker_id,
            target,
        )
    }

    fn cache_job(&mut self, event: &MintBlockEvent, now: Instant) {
        let job_id = JobId::from_bob(&event.minting_blob).job_id;

        self.recent_jobs.insert(job_id, JobCacheEntry { ts: now });
        self.recent_job_queue.push_back(job_id);
        self.recent_job_queue
            .retain(|id| self.recent_jobs.contains_key(id));

        while self.recent_job_queue.len() > MAX_RECENT_JOBS {
            if let Some(id) = self.recent_job_queue.pop_front() {
                self.recent_jobs.remove(&id);
            }
        }

        self.recent_jobs.retain(|_, entry| {
            now.duration_since(entry.ts) <= Duration::from_secs(self.limits.stale_window_secs)
        });
        self.solved_jobs.retain(|_, ts| {
            now.duration_since(*ts) <= Duration::from_secs(self.limits.stale_window_secs)
        });
    }

    fn update_job(
        &mut self,
        event: MintBlockEvent,
        now: Instant,
    ) -> Vec<(WorkerId, mpsc::Sender<String>, String)> {
        self.current_job = Some(event.clone());
        self.cache_job(&event, now);

        let mut pending = Vec::new();
        for (id, session) in &self.workers {
            let _ = session
                .worker
                .diff_manager
                .write()
                .unwrap()
                .maybe_decay(&session.worker.base_info.login);
            let resp = Self::get_downstream_job(&session.worker, false, &event);
            let notif = JsonRpcNotification {
                jsonrpc: Some("2.0"),
                method: "job",
                params: resp.job,
            };
            match serde_json::to_string(&notif) {
                Ok(msg) => pending.push((*id, session.out_tx.clone(), msg)),
                Err(err) => {
                    debug!(target: "stratum_server", "serialize job notification failed: {}", err)
                }
            }
        }
        pending
    }

    fn is_current_job(&self, event: &MintBlockEvent) -> bool {
        self.current_job.as_ref().is_some_and(|current| {
            JobId::from_bob(&current.minting_blob) == JobId::from_bob(&event.minting_blob)
        })
    }

    fn record_error(
        counters: &mut HashMap<WorkerId, ErrorCounter>,
        worker_id: WorkerId,
        now: Instant,
        threshold: u32,
    ) -> bool {
        let entry = counters.entry(worker_id).or_insert(ErrorCounter {
            first: now,
            count: 0,
        });
        if now.duration_since(entry.first) > Duration::from_secs(ERROR_WINDOW_SECS) {
            entry.first = now;
            entry.count = 1;
        } else {
            entry.count = entry.count.saturating_add(1);
        }
        entry.count >= threshold
    }

    fn record_invalid_share(&mut self, worker_id: WorkerId, now: Instant) -> bool {
        Self::record_error(
            &mut self.invalid_share_counters,
            worker_id,
            now,
            self.limits.max_invalid_shares,
        )
    }

    fn record_job_miss(&mut self, worker_id: WorkerId, now: Instant) -> bool {
        Self::record_error(
            &mut self.job_miss_counters,
            worker_id,
            now,
            self.limits.max_job_misses,
        )
    }

    fn record_stale_share(&mut self, worker_id: WorkerId, now: Instant) -> bool {
        Self::record_error(
            &mut self.stale_share_counters,
            worker_id,
            now,
            self.limits.max_stale_shares,
        )
    }

    fn is_duplicate_share(&mut self, worker_id: WorkerId, key: ShareKey, now: Instant) -> bool {
        let map = self.share_dedup.entry(worker_id).or_default();
        map.retain(|_, ts| {
            now.duration_since(*ts) <= Duration::from_secs(self.limits.share_dedup_window_secs)
        });
        if let std::collections::hash_map::Entry::Vacant(entry) = map.entry(key) {
            entry.insert(now);
            false
        } else {
            true
        }
    }

    fn check_rate_limit(&mut self, worker_id: WorkerId, now: Instant) -> bool {
        let entry = self.share_rate.entry(worker_id).or_insert(RateCounter {
            first: now,
            count: 0,
        });
        if now.duration_since(entry.first) > Duration::from_secs(self.limits.share_rate_window_secs)
        {
            entry.first = now;
            entry.count = 1;
            return false;
        }
        entry.count = entry.count.saturating_add(1);
        entry.count > self.limits.max_shares_per_window
    }

    fn mark_solved_job(&mut self, job_id: [u8; 8], now: Instant) {
        self.solved_jobs.insert(job_id, now);
        self.solved_jobs.retain(|_, ts| {
            now.duration_since(*ts) <= Duration::from_secs(self.limits.stale_window_secs)
        });
    }

    fn is_solved_job(&mut self, job_id: [u8; 8], now: Instant) -> bool {
        self.solved_jobs.retain(|_, ts| {
            now.duration_since(*ts) <= Duration::from_secs(self.limits.stale_window_secs)
        });
        self.solved_jobs.contains_key(&job_id)
    }

    fn build_mining_blob(event: &MintBlockEvent, extra: &BlockHeaderExtra) -> Result<Vec<u8>> {
        let mut blob = event.minting_blob.clone();
        if blob.len() < 39 {
            return Err(anyhow::anyhow!("invalid minting blob"));
        }
        blob[35..39].copy_from_slice(extra.as_slice());
        Ok(blob)
    }

    fn is_hex(s: &str) -> bool {
        s.bytes().all(|b| b.is_ascii_hexdigit())
    }

    fn validate_share_params(
        &mut self,
        worker_id: WorkerId,
        share: &ShareRequest,
        now: Instant,
    ) -> SubmitShareResponse {
        if share.id.len() != WORKER_ID_HEX_LEN || !Self::is_hex(&share.id) {
            let disconnect = self.record_invalid_share(worker_id, now);
            if disconnect {
                self.log_disconnect(worker_id, "invalid worker id");
                self.drop_worker_state(worker_id);
            }
            return Self::reject(-1, "invalid worker id", disconnect);
        }

        if share.job_id.len() != JOB_ID_HEX_LEN || !Self::is_hex(&share.job_id) {
            let disconnect = self.record_invalid_share(worker_id, now);
            if disconnect {
                self.log_disconnect(worker_id, "invalid job id");
                self.drop_worker_state(worker_id);
            }
            return Self::reject(-1, "invalid job id", disconnect);
        }

        if share.nonce.is_empty()
            || share.nonce.len() > MAX_NONCE_HEX_LEN
            || !Self::is_hex(&share.nonce)
        {
            let disconnect = self.record_invalid_share(worker_id, now);
            if disconnect {
                self.log_disconnect(worker_id, "invalid nonce");
                self.drop_worker_state(worker_id);
            }
            return Self::reject(-1, "invalid nonce", disconnect);
        }

        Self::accept()
    }

    fn note_stat(&mut self, now: Instant) {
        if now.duration_since(self.last_stats_log) >= Duration::from_secs(STATS_LOG_INTERVAL_SECS) {
            info!(
                target: "stratum_server",
                "share_stats accepted:{} invalid:{} duplicate:{} stale:{} job_miss:{} rate_limited:{} forwarded_blocks:{}",
                self.stats.accepted,
                self.stats.invalid,
                self.stats.duplicate,
                self.stats.stale,
                self.stats.job_miss,
                self.stats.rate_limited,
                self.forwarded_blocks
            );
            self.last_stats_log = now;
        }
    }

    fn reject(code: i32, message: &str, disconnect: bool) -> SubmitShareResponse {
        SubmitShareResponse::Rejected {
            code,
            message: message.to_string(),
            disconnect,
        }
    }

    fn accept() -> SubmitShareResponse {
        SubmitShareResponse::Accepted
    }

    fn log_disconnect(&self, worker_id: WorkerId, reason: &str) {
        warn!(
            target: "stratum_server",
            "disconnect worker {}: {}",
            worker_id.to_hex(),
            reason
        );
    }
}

struct PplnsRuntime {
    config: StratumPplnsConfig,
    store: Box<dyn PplnsStore>,
    dirty_ops: u64,
    local_share_anchor_map: BTreeMap<u64, u64>,
    last_batch_run_millis: Option<u64>,
    last_settled_height: Option<u64>,
}

impl PplnsRuntime {
    fn new(config: StratumPplnsConfig) -> Result<Self> {
        let mut store = build_pplns_store(&config)?;
        let last_batch_run_millis = store.last_batch_run_millis()?;
        let last_settled_height = store.last_settled_height()?;
        Ok(Self {
            config,
            store,
            dirty_ops: 0,
            local_share_anchor_map: BTreeMap::new(),
            last_batch_run_millis,
            last_settled_height,
        })
    }

    fn ingest_enabled(&self) -> bool {
        self.config.enabled && self.config.ingest_enabled
    }

    fn settlement_enabled(&self) -> bool {
        self.config.enabled && self.config.settlement_enabled
    }

    fn remember_share_anchor(&mut self, local_seq: u64, persisted_seq: u64) {
        self.local_share_anchor_map.insert(local_seq, persisted_seq);
        while self.local_share_anchor_map.len() > 131_072 {
            let _ = self.local_share_anchor_map.pop_first();
        }
    }

    fn resolve_anchor_share_seq(&self, local_seq: u64) -> u64 {
        self.local_share_anchor_map
            .get(&local_seq)
            .copied()
            .unwrap_or(local_seq)
    }

    fn should_run_batch(&self, now_millis: u64) -> bool {
        match self.last_batch_run_millis {
            None => true,
            Some(last) => {
                let period_millis = self.config.batch_period_secs.saturating_mul(1_000);
                now_millis.saturating_sub(last) >= period_millis
            }
        }
    }

    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis() as u64)
    }

    fn flush_store(&mut self, force: bool) {
        if !force && self.dirty_ops < SHARE_FLUSH_BATCH_SIZE {
            return;
        }
        match self.store.persist() {
            Ok(_) => {
                self.dirty_ops = 0;
            }
            Err(err) => {
                error!(target: "stratum_server", "pplns persist failed: {}", err);
            }
        }
    }

    fn on_accepted_share(
        &mut self,
        local_seq: u64,
        worker_id: String,
        account: String,
        difficulty: u64,
        accepted_at_millis: u64,
    ) {
        if !self.ingest_enabled() {
            return;
        }
        let persisted_seq = match self.store.append_share(
            ShareRecord {
                seq: local_seq,
                account,
                worker_id,
                difficulty: difficulty.max(1),
                accepted_at_millis,
            },
            self.config.max_retained_shares,
        ) {
            Ok(seq) => seq,
            Err(err) => {
                warn!(target: "stratum_server", "pplns append share failed: {}", err);
                return;
            }
        };
        self.remember_share_anchor(local_seq, persisted_seq);
        self.dirty_ops = self.dirty_ops.saturating_add(1);
        self.flush_store(false);
    }

    fn on_candidate_submit(
        &mut self,
        job_id: String,
        nonce: u32,
        extra: String,
        account: String,
        worker_id: String,
        anchor_share_seq: u64,
        expected_block_number: u64,
        submitted_at_millis: u64,
    ) {
        if !self.ingest_enabled() {
            return;
        }
        let anchor_share_seq = self.resolve_anchor_share_seq(anchor_share_seq);
        if let Err(err) = self.store.upsert_pending_submit(
            PendingSubmitRecord {
                job_id,
                nonce,
                extra,
                account,
                worker_id,
                anchor_share_seq,
                expected_block_number,
                submitted_at_millis,
            },
            self.config.max_retained_candidates,
        ) {
            warn!(
                target: "stratum_server",
                "pplns upsert pending submit failed: {}",
                err
            );
            return;
        }
        self.dirty_ops = self.dirty_ops.saturating_add(1);
        self.flush_store(false);
    }

    fn on_candidate_solved(
        &mut self,
        job_id: String,
        nonce: u32,
        extra: String,
        block_hash: HashValue,
        block_number: u64,
    ) {
        if !self.ingest_enabled() {
            return;
        }
        let pending = match self.store.take_pending_submit(&job_id, nonce, &extra) {
            Ok(pending) => pending,
            Err(err) => {
                warn!(
                    target: "stratum_server",
                    "pplns take pending submit failed: {}",
                    err
                );
                return;
            }
        };

        let Some(pending) = pending else {
            debug!(
                target: "stratum_server",
                "pplns solved event missing pending submit: job_id={}, nonce={}, extra={}",
                job_id,
                nonce,
                extra
            );
            return;
        };

        if let Err(err) = self.store.upsert_candidate(
            CandidateRecord {
                block_hash: block_hash.to_string(),
                block_number,
                account: pending.account,
                worker_id: pending.worker_id,
                anchor_share_seq: pending.anchor_share_seq,
                found_at_millis: Self::now_millis(),
                status: CandidateStatus::Pending,
                reward: None,
                settled_at_millis: None,
            },
            self.config.max_retained_candidates,
        ) {
            warn!(target: "stratum_server", "pplns upsert candidate failed: {}", err);
            return;
        }
        self.dirty_ops = self.dirty_ops.saturating_add(1);
        self.flush_store(true);
    }

    async fn settle_tick(&mut self, rpc: &AsyncRpcClient) -> Result<()> {
        if !self.settlement_enabled() {
            return Ok(());
        }
        let now = Self::now_millis();
        if !self.should_run_batch(now) {
            return Ok(());
        }
        let acquired = self.store.try_acquire_settlement_lock()?;
        if !acquired {
            return Ok(());
        }

        let settlement_result = self.settle_pending(rpc).await;
        let release_result = self.store.release_settlement_lock();

        match settlement_result {
            Ok((settled, orphaned)) => {
                if let Err(err) = self.store.set_last_batch_run_millis(now) {
                    warn!(
                        target: "stratum_server",
                        "pplns persist batch checkpoint failed: {}",
                        err
                    );
                } else {
                    self.last_batch_run_millis = Some(now);
                    self.dirty_ops = self.dirty_ops.saturating_add(1);
                    self.flush_store(true);
                    if settled > 0 || orphaned > 0 {
                        debug!(
                            target: "stratum_server",
                            "pplns settled={}, orphaned={}, balances={}",
                            settled,
                            orphaned,
                            self.store.balances_len().unwrap_or_default()
                        );
                    }
                }
            }
            Err(err) => {
                warn!(target: "stratum_server", "pplns settle tick failed: {}", err);
            }
        }

        if let Err(err) = release_result {
            warn!(
                target: "stratum_server",
                "pplns release settlement lock failed: {}",
                err
            );
        }

        Ok(())
    }

    async fn settle_pending(&mut self, rpc: &AsyncRpcClient) -> Result<(u64, u64)> {
        let head = rpc.chain_info().await?;
        let head_number = head.head.number.0;
        let confirmed_head =
            head_number.saturating_sub(self.config.confirmations.saturating_sub(1));
        let mut next_height = self.last_settled_height.unwrap_or(0).saturating_add(1);
        if next_height > confirmed_head {
            return Ok((0, 0));
        }

        let mut settled_count = 0u64;
        let mut orphaned_count = 0u64;

        while next_height <= confirmed_head {
            let Some(main_block) = rpc.chain_get_block_by_number(next_height, None).await? else {
                break;
            };
            let main_block_hash = main_block.header.block_hash;
            let mut reward_for_height: Option<u128> = None;
            let candidates = self.store.pending_candidates_by_height(next_height)?;
            for candidate in candidates {
                let candidate_hash = match HashValue::from_str(candidate.block_hash.as_str()) {
                    Ok(hash) => hash,
                    Err(err) => {
                        warn!(
                            target: "stratum_server",
                            "invalid candidate hash {}: {}",
                            candidate.block_hash,
                            err
                        );
                        if self
                            .store
                            .mark_candidate_orphaned(&candidate.block_hash, Self::now_millis())?
                        {
                            orphaned_count = orphaned_count.saturating_add(1);
                        }
                        continue;
                    }
                };
                if candidate_hash != main_block_hash {
                    if self
                        .store
                        .mark_candidate_orphaned(&candidate.block_hash, Self::now_millis())?
                    {
                        orphaned_count = orphaned_count.saturating_add(1);
                    }
                    continue;
                }

                let reward = match reward_for_height {
                    Some(reward) => reward,
                    None => {
                        let reward =
                            Self::fetch_block_reward(rpc, main_block_hash, next_height).await?;
                        reward_for_height = Some(reward);
                        reward
                    }
                };

                let window_shares = self
                    .store
                    .window_shares(candidate.anchor_share_seq, self.config.window_shares)?;
                let credits = Self::allocate_credits(&candidate, &window_shares, reward);
                if self.store.finalize_confirmed_candidate(
                    &candidate.block_hash,
                    reward,
                    Self::now_millis(),
                    credits,
                )? {
                    settled_count = settled_count.saturating_add(1);
                }
            }

            self.store.set_last_settled_height(next_height)?;
            self.last_settled_height = Some(next_height);
            next_height = next_height.saturating_add(1);
        }

        if settled_count > 0 || orphaned_count > 0 {
            let prune_before =
                confirmed_head.saturating_sub(self.config.confirmations.saturating_mul(2));
            self.store
                .remove_confirmed_below(prune_before, self.config.max_retained_candidates)?;
        }
        Ok((settled_count, orphaned_count))
    }

    async fn fetch_block_reward(
        rpc: &AsyncRpcClient,
        block_hash: HashValue,
        block_number: u64,
    ) -> Result<u128> {
        let txn_infos = rpc.chain_get_block_txn_infos(block_hash).await?;
        let mut reward = 0u128;
        for txn_info in txn_infos {
            let txn_hash = txn_info.transaction_hash;
            let events = rpc.chain_get_events_by_txn_hash(txn_hash, None).await?;
            for event_info in events {
                let event = event_info.event;
                if event.block_hash != Some(block_hash) {
                    continue;
                }
                let tag = event.type_tag.to_string();
                let data = &event.data.0;
                if let Some((reward_block_number, amount)) =
                    Self::parse_block_reward_event(&tag, data)
                {
                    if reward_block_number == block_number {
                        reward = reward.saturating_add(amount);
                    }
                }
            }
        }
        Ok(reward)
    }

    fn parse_block_reward_event(tag: &str, data: &[u8]) -> Option<(u64, u128)> {
        if !tag.contains("BlockRewardEvent") {
            return None;
        }
        if let Ok(reward) = BlockRewardEventV1::try_from_bytes(data) {
            return Some((
                reward.block_number,
                reward.block_reward.saturating_add(reward.gas_fees),
            ));
        }
        if let Ok(reward) = BlockRewardEventV2::try_from_bytes(data) {
            return Some((
                reward.block_number,
                reward.block_reward.saturating_add(reward.gas_fees),
            ));
        }
        None
    }

    fn allocate_credits(
        candidate: &CandidateRecord,
        shares: &[ShareRecord],
        reward: u128,
    ) -> HashMap<String, u128> {
        let mut credits = HashMap::new();
        if reward == 0 {
            return credits;
        }
        if shares.is_empty() {
            credits.insert(candidate.account.clone(), reward);
            return credits;
        }
        let total_weight: u128 = shares
            .iter()
            .map(|share| u128::from(share.difficulty.max(1)))
            .sum();
        if total_weight == 0 {
            credits.insert(candidate.account.clone(), reward);
            return credits;
        }

        let mut distributed = 0u128;
        for share in shares {
            let weight = u128::from(share.difficulty.max(1));
            let amount = reward.saturating_mul(weight) / total_weight;
            if amount == 0 {
                continue;
            }
            let entry = credits.entry(share.account.clone()).or_default();
            *entry = entry.saturating_add(amount);
            distributed = distributed.saturating_add(amount);
        }
        let remainder = reward.saturating_sub(distributed);
        if remainder > 0 {
            let entry = credits.entry(candidate.account.clone()).or_default();
            *entry = entry.saturating_add(remainder);
        }
        credits
    }
}

#[derive(Clone)]
struct ForwardSubmit {
    job_id_hex: String,
    job_id: [u8; 8],
    minting_blob: String,
    nonce: u32,
    extra: String,
    expected_block_number: u64,
}

struct AcceptedShareIngest {
    seq: u64,
    worker_id: String,
    account: String,
    difficulty: u64,
    accepted_at_millis: u64,
}

struct CandidateSubmitIngest {
    job_id: String,
    nonce: u32,
    extra: String,
    account: String,
    worker_id: String,
    anchor_share_seq: u64,
    expected_block_number: u64,
    submitted_at_millis: u64,
}

#[derive(Clone)]
struct App {
    rpc: Arc<AsyncRpcClient>,
    state: Arc<Mutex<GatewayState>>,
    job_poll: Duration,
    pplns: Option<Arc<Mutex<PplnsRuntime>>>,
}

impl App {
    async fn ensure_current_job(&self) -> Result<()> {
        {
            let state = self.state.lock().await;
            if state.current_job.is_some() {
                return Ok(());
            }
        }

        match self.rpc.miner_get_job().await {
            Ok(Some(job)) => {
                self.on_new_job(job).await;
                Ok(())
            }
            Ok(None) => Err(anyhow::anyhow!("no job")),
            Err(err) => Err(anyhow::anyhow!("fetch job failed: {}", err)),
        }
    }

    async fn on_new_job(&self, event: MintBlockEvent) {
        let now = Instant::now();
        let pending = {
            let mut state = self.state.lock().await;
            if state.is_current_job(&event) {
                return;
            }
            state.update_job(event, now)
        };

        let mut failed = Vec::new();
        for (worker_id, mut tx, msg) in pending {
            if tx.send(msg).await.is_err() {
                failed.push(worker_id);
            }
        }

        if !failed.is_empty() {
            let mut state = self.state.lock().await;
            for id in failed {
                state.drop_worker_state(id);
            }
        }
    }

    async fn register_worker(
        &self,
        login: LoginRequest,
        out_tx: mpsc::Sender<String>,
    ) -> Result<(String, StratumJobResponse)> {
        self.ensure_current_job().await?;
        let (worker_id, mut job) = {
            let mut state = self.state.lock().await;
            state.register_worker(login, out_tx)?
        };
        job.login = None;
        Ok((worker_id.to_hex(), job))
    }

    async fn unregister_worker_hex(&self, worker_id: &str) {
        if let Ok(worker_id) = WorkerId::from_hex(worker_id.to_string()) {
            let mut state = self.state.lock().await;
            state.drop_worker_state(worker_id);
        }
    }

    async fn mark_solved_job(&self, job_id: [u8; 8]) {
        let mut state = self.state.lock().await;
        state.mark_solved_job(job_id, Instant::now());
        state.forwarded_blocks = state.forwarded_blocks.saturating_add(1);
    }

    async fn ingest_accepted_share(&self, event: AcceptedShareIngest) {
        let Some(pplns) = &self.pplns else {
            return;
        };
        let mut pplns = pplns.lock().await;
        pplns.on_accepted_share(
            event.seq,
            event.worker_id,
            event.account,
            event.difficulty,
            event.accepted_at_millis,
        );
    }

    async fn ingest_candidate_submit(&self, event: CandidateSubmitIngest) {
        let Some(pplns) = &self.pplns else {
            return;
        };
        let mut pplns = pplns.lock().await;
        pplns.on_candidate_submit(
            event.job_id,
            event.nonce,
            event.extra,
            event.account,
            event.worker_id,
            event.anchor_share_seq,
            event.expected_block_number,
            event.submitted_at_millis,
        );
    }

    async fn ingest_candidate_solved(
        &self,
        job_id: String,
        nonce: u32,
        extra: String,
        block_hash: HashValue,
        block_number: u64,
    ) {
        let Some(pplns) = &self.pplns else {
            return;
        };
        let mut pplns = pplns.lock().await;
        pplns.on_candidate_solved(job_id, nonce, extra, block_hash, block_number);
    }

    fn spawn_forward_submit(&self, req: ForwardSubmit) {
        let app = self.clone();
        tokio::spawn(async move {
            let submit_extra = req.extra.clone();
            let submit_res = app
                .rpc
                .miner_submit(req.minting_blob, req.nonce, submit_extra)
                .await;
            match submit_res {
                Ok(view) => {
                    app.mark_solved_job(req.job_id).await;
                    app.ingest_candidate_solved(
                        req.job_id_hex,
                        req.nonce,
                        req.extra,
                        view.block_hash,
                        req.expected_block_number,
                    )
                    .await;
                }
                Err(err) => {
                    warn!(target: "stratum_server", "forward submit failed: {}", err);
                }
            }
        });
    }

    async fn submit_share(&self, share: ShareRequest) -> SubmitShareResponse {
        let now = Instant::now();
        let mut forward: Option<ForwardSubmit> = None;
        let accepted_share_ingest: AcceptedShareIngest;
        let mut candidate_submit_ingest: Option<CandidateSubmitIngest> = None;

        {
            let mut state = self.state.lock().await;

            let worker_id = match WorkerId::from_hex(share.id.clone()) {
                Ok(worker_id) => worker_id,
                Err(err) => {
                    warn!(target: "stratum_server", "invalid worker id: {}", err);
                    state.stats.invalid = state.stats.invalid.saturating_add(1);
                    state.note_stat(now);
                    return GatewayState::reject(-1, "invalid worker id", false);
                }
            };

            if !state.workers.contains_key(&worker_id) {
                state.stats.job_miss = state.stats.job_miss.saturating_add(1);
                state.note_stat(now);
                state.log_disconnect(worker_id, "worker not found");
                state.drop_worker_state(worker_id);
                return GatewayState::reject(-1, "worker not found", true);
            }

            let validation = state.validate_share_params(worker_id, &share, now);
            if !matches!(validation, SubmitShareResponse::Accepted) {
                state.stats.invalid = state.stats.invalid.saturating_add(1);
                state.note_stat(now);
                return validation;
            }

            if state.check_rate_limit(worker_id, now) {
                state.stats.rate_limited = state.stats.rate_limited.saturating_add(1);
                state.note_stat(now);
                return GatewayState::reject(-1, "rate limited", false);
            }

            let current_mint_event = match state.current_job.clone() {
                Some(event) => event,
                None => {
                    let disconnect = state.record_job_miss(worker_id, now);
                    state.stats.job_miss = state.stats.job_miss.saturating_add(1);
                    state.note_stat(now);
                    if disconnect {
                        state.log_disconnect(worker_id, "job not found");
                        state.drop_worker_state(worker_id);
                    }
                    return GatewayState::reject(-1, "job not found", disconnect);
                }
            };

            let job_id = match JobId::new(&share.job_id) {
                Ok(job_id) => job_id,
                Err(err) => {
                    warn!(target: "stratum_server", "invalid job id: {}", err);
                    let disconnect = state.record_invalid_share(worker_id, now);
                    state.stats.invalid = state.stats.invalid.saturating_add(1);
                    state.note_stat(now);
                    if disconnect {
                        state.log_disconnect(worker_id, "invalid job id");
                        state.drop_worker_state(worker_id);
                    }
                    return GatewayState::reject(-1, "invalid job id", disconnect);
                }
            };

            let (worker_login, desired_diff) = match state.workers.get(&worker_id) {
                Some(session) => {
                    let diff = session.worker.diff_manager.read().unwrap().difficulty;
                    (session.worker.base_info.login.clone(), diff)
                }
                None => {
                    let disconnect = state.record_job_miss(worker_id, now);
                    if disconnect {
                        state.log_disconnect(worker_id, "worker not found");
                        state.drop_worker_state(worker_id);
                    }
                    return GatewayState::reject(-1, "worker not found", disconnect);
                }
            };
            let worker_account = GatewayState::account_from_login(&worker_login);

            let parsed = match parse_share_nonce_extra(&share) {
                Ok(parsed) => parsed,
                Err(err) => {
                    warn!(target: "stratum_server", "invalid share: {}", err);
                    let disconnect = state.record_invalid_share(worker_id, now);
                    state.stats.invalid = state.stats.invalid.saturating_add(1);
                    state.note_stat(now);
                    if disconnect {
                        state.log_disconnect(worker_id, "invalid share");
                        state.drop_worker_state(worker_id);
                    }
                    return GatewayState::reject(-1, "invalid share", disconnect);
                }
            };

            let share_key = ShareKey {
                job_id: job_id.job_id,
                nonce: parsed.nonce,
                extra: parsed.extra,
            };
            if state.is_duplicate_share(worker_id, share_key, now) {
                let disconnect = state.record_invalid_share(worker_id, now);
                state.stats.duplicate = state.stats.duplicate.saturating_add(1);
                state.note_stat(now);
                if disconnect {
                    state.log_disconnect(worker_id, "duplicate share");
                    state.drop_worker_state(worker_id);
                }
                return GatewayState::reject(-1, "duplicate share", disconnect);
            }

            let submit_job_id = JobId::from_bob(&current_mint_event.minting_blob);
            if job_id != submit_job_id {
                let job_known = state.recent_jobs.contains_key(&job_id.job_id);
                let (disconnect, reason) = if job_known {
                    state.stats.stale = state.stats.stale.saturating_add(1);
                    (state.record_stale_share(worker_id, now), "stale share")
                } else {
                    state.stats.job_miss = state.stats.job_miss.saturating_add(1);
                    (state.record_job_miss(worker_id, now), "job not found")
                };
                debug!(
                    target: "stratum_server",
                    "received job mismatch with current job, submitted={:?}, current={:?}, worker={}, known_job={}",
                    job_id,
                    submit_job_id,
                    share.id,
                    job_known
                );
                state.note_stat(now);
                if disconnect {
                    state.log_disconnect(worker_id, reason);
                    state.drop_worker_state(worker_id);
                }
                return GatewayState::reject(-1, reason, disconnect);
            }

            let extra = BlockHeaderExtra::new(parsed.extra);
            let mining_blob = match GatewayState::build_mining_blob(&current_mint_event, &extra) {
                Ok(blob) => blob,
                Err(err) => {
                    warn!(target: "stratum_server", "build mining blob failed: {}", err);
                    state.stats.invalid = state.stats.invalid.saturating_add(1);
                    state.note_stat(now);
                    return GatewayState::reject(-1, "invalid blob", false);
                }
            };

            let pow_hash = match current_mint_event.strategy.calculate_pow_hash(
                &mining_blob,
                parsed.nonce,
                &extra,
            ) {
                Ok(hash) => hash,
                Err(err) => {
                    warn!(target: "stratum_server", "calculate pow hash failed: {}", err);
                    state.stats.invalid = state.stats.invalid.saturating_add(1);
                    state.note_stat(now);
                    return GatewayState::reject(-1, "invalid share", false);
                }
            };
            let pow_hash_u256: U256 = pow_hash.into();

            let network = current_mint_event.difficulty;
            let effective = if desired_diff > network {
                network
            } else {
                desired_diff
            };

            let share_target = match difficult_to_target(effective) {
                Ok(target) => target,
                Err(err) => {
                    warn!(target: "stratum_server", "calculate share target failed: {}", err);
                    return GatewayState::reject(-1, "internal error", false);
                }
            };

            if pow_hash_u256 > share_target {
                let disconnect = state.record_invalid_share(worker_id, now);
                state.stats.invalid = state.stats.invalid.saturating_add(1);
                state.note_stat(now);
                if disconnect {
                    state.log_disconnect(worker_id, "low difficulty share");
                    state.drop_worker_state(worker_id);
                }
                return GatewayState::reject(-1, "low difficulty share", disconnect);
            }

            if let Some(session) = state.workers.get_mut(&worker_id) {
                let _ = session
                    .worker
                    .diff_manager
                    .write()
                    .unwrap()
                    .try_update(worker_login);
            }

            let network_target = match difficult_to_target(current_mint_event.difficulty) {
                Ok(target) => target,
                Err(err) => {
                    warn!(target: "stratum_server", "calculate network target failed: {}", err);
                    return GatewayState::reject(-1, "internal error", false);
                }
            };

            if pow_hash_u256 <= network_target && !state.is_solved_job(job_id.job_id, now) {
                forward = Some(ForwardSubmit {
                    job_id_hex: share.job_id.clone(),
                    job_id: job_id.job_id,
                    minting_blob: hex::encode(current_mint_event.minting_blob),
                    nonce: parsed.nonce,
                    extra: hex::encode(parsed.extra),
                    expected_block_number: current_mint_event.block_number,
                });
            }

            let share_seq = state.next_share_seq();
            let accepted_at_millis = now_millis();
            accepted_share_ingest = AcceptedShareIngest {
                seq: share_seq,
                worker_id: worker_id.to_hex(),
                account: worker_account.clone(),
                difficulty: effective.as_u64(),
                accepted_at_millis,
            };

            if let Some(req) = forward.as_ref() {
                candidate_submit_ingest = Some(CandidateSubmitIngest {
                    job_id: req.job_id_hex.clone(),
                    nonce: req.nonce,
                    extra: req.extra.clone(),
                    account: worker_account,
                    worker_id: worker_id.to_hex(),
                    anchor_share_seq: share_seq,
                    expected_block_number: req.expected_block_number,
                    submitted_at_millis: accepted_at_millis,
                });
            }

            state.stats.accepted = state.stats.accepted.saturating_add(1);
            state.note_stat(now);
        }

        self.ingest_accepted_share(accepted_share_ingest).await;
        if let Some(event) = candidate_submit_ingest {
            self.ingest_candidate_submit(event).await;
        }
        if let Some(req) = forward {
            self.spawn_forward_submit(req);
        }

        GatewayState::accept()
    }

    async fn run_job_poll_loop(self) {
        loop {
            match self.rpc.miner_get_job().await {
                Ok(Some(job)) => self.on_new_job(job).await,
                Ok(None) => {
                    debug!(target: "stratum_server", "upstream mining.get_job returned none")
                }
                Err(err) => {
                    warn!(target: "stratum_server", "upstream mining.get_job failed: {}", err)
                }
            }
            sleep(self.job_poll).await;
        }
    }

    async fn run_pplns_settlement_loop(self, interval_secs: u64) {
        let interval = Duration::from_secs(interval_secs.max(1));
        loop {
            if let Some(pplns) = &self.pplns {
                let mut pplns = pplns.lock().await;
                if let Err(err) = pplns.settle_tick(&self.rpc).await {
                    warn!(target: "stratum_server", "pplns settlement tick failed: {}", err);
                }
            }
            sleep(interval).await;
        }
    }
}

#[derive(Debug)]
struct ParsedShare {
    nonce: u32,
    extra: [u8; 4],
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis() as u64)
}

fn parse_share_nonce_extra(share: &ShareRequest) -> Result<ParsedShare> {
    let nonce_temp = u32::from_str_radix(share.nonce.as_str(), 16)?;
    let mut nonce_bytes = Vec::new();
    let _ = nonce_bytes.write_u32::<LittleEndian>(nonce_temp);
    let nonce = BigEndian::read_u32(&nonce_bytes);

    let extra_raw = hex::decode(&share.id)?;
    let extra: [u8; 4] = extra_raw
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to parse extra"))?;

    Ok(ParsedShare { nonce, extra })
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

#[tokio::main]
async fn main() -> Result<()> {
    let _logger = starcoin_logger::init();
    let opt = Opt::parse();

    let limits = StratumLimits {
        share_dedup_window_secs: opt.share_dedup_window_secs,
        stale_window_secs: opt.stale_window_secs,
        share_rate_window_secs: opt.share_rate_window_secs,
        max_shares_per_window: opt.max_shares_per_window,
        max_invalid_shares: opt.max_invalid_shares,
        max_job_misses: opt.max_job_misses,
        max_stale_shares: opt.max_stale_shares,
        max_workers_per_account: opt.max_workers_per_account,
    };

    let conn = parse_conn_source(&opt.node_rpc)?;
    let rpc = Arc::new(AsyncRpcClient::new(conn).await?);
    let pplns = build_pplns_runtime(&opt)?;
    let app = App {
        rpc,
        state: Arc::new(Mutex::new(GatewayState::new(limits))),
        job_poll: Duration::from_millis(opt.job_poll_ms.max(100)),
        pplns,
    };

    let poll_app = app.clone();
    tokio::spawn(async move {
        poll_app.run_job_poll_loop().await;
    });
    if let Some(pplns) = app.pplns.as_ref() {
        let settle_enabled = {
            let pplns = pplns.lock().await;
            pplns.settlement_enabled()
        };
        if settle_enabled {
            let settlement_app = app.clone();
            let interval_secs = opt.pplns_settlement_interval_secs;
            tokio::spawn(async move {
                settlement_app
                    .run_pplns_settlement_loop(interval_secs)
                    .await;
            });
        }
    }

    run_stratum_server(opt.listen, app).await
}

fn parse_conn_source(node_rpc: &str) -> Result<ConnSource> {
    match Connect::from_str(node_rpc)? {
        Connect::WebSocket(url) => Ok(ConnSource::WebSocket(url)),
        Connect::IPC(Some(path)) => Ok(ConnSource::Ipc(path)),
        Connect::IPC(None) => Err(anyhow::anyhow!(
            "node rpc ipc path is empty, please set --node-rpc <path-to-ipc-file>"
        )),
    }
}

fn build_pplns_runtime(opt: &Opt) -> Result<Option<Arc<Mutex<PplnsRuntime>>>> {
    if !opt.pplns_enabled {
        return Ok(None);
    }
    let config = StratumPplnsConfig {
        enabled: true,
        ingest_enabled: true,
        settlement_enabled: true,
        window_shares: opt.pplns_window_shares.max(1),
        confirmations: opt.pplns_confirmations.max(1),
        settlement_interval_secs: opt.pplns_settlement_interval_secs.max(1),
        batch_period_secs: opt.pplns_batch_period_secs.max(60),
        max_retained_shares: opt
            .pplns_max_retained_shares
            .max(opt.pplns_window_shares.max(1)),
        max_retained_candidates: opt.pplns_max_retained_candidates.max(64),
        database_url: opt.pplns_database_url.clone(),
    };
    let runtime = PplnsRuntime::new(config)?;
    info!(
        target: "stratum_server",
        "pplns enabled: ingest={}, settlement={}, window_shares={}, confirmations={}, interval_secs={}, batch_period_secs={}",
        runtime.ingest_enabled(),
        runtime.settlement_enabled(),
        runtime.config.window_shares,
        runtime.config.confirmations,
        runtime.config.settlement_interval_secs,
        runtime.config.batch_period_secs
    );
    Ok(Some(Arc::new(Mutex::new(runtime))))
}

async fn run_stratum_server(address: SocketAddr, app: App) -> Result<()> {
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
                warn!(
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

        if req_rate.exceeded() {
            disconnect_reason = Some("request rate limit exceeded".to_string());
            break;
        }

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
                                warn!(
                                    target: "stratum_server",
                                    "send submit success response failed: {}",
                                    err
                                );
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
        warn!(
            target: "stratum_server",
            "disconnect client: peer={}, worker={}, logged_in={}, reason={}",
            peer_addr,
            worker_id.clone().unwrap_or_else(|| "-".to_string()),
            logged_in,
            reason
        );
    }

    if let Some(worker_id) = worker_id {
        app.unregister_worker_hex(&worker_id).await;
    }
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
