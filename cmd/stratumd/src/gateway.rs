use anyhow::Result;
use byteorder::{BigEndian, ByteOrder, LittleEndian, WriteBytesExt};
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use starcoin_consensus::{difficult_to_target, Consensus};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_stratumd::diff_manager::DifficultyManager;
use starcoin_stratumd::node_rpc::NodeRpc;
use starcoin_stratumd::pplns::PplnsRuntime;
use starcoin_stratumd::stratum_rpc::{
    JobId, LoginRequest, MinerWorker, ShareRequest, StratumJobResponse, SubmitShareResponse,
    WorkerId,
};
use starcoin_stratumd::{
    difficulty_to_target_hex, target_hex_to_difficulty, StratumLimits, ERROR_WINDOW_SECS,
    JOB_ID_HEX_LEN, MAX_NONCE_HEX_LEN, MAX_RECENT_JOBS, STATS_LOG_INTERVAL_SECS, TARGET_HEX_LEN,
    WORKER_ID_HEX_LEN,
};
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::U256;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;

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
    assigned_diff: U256,
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
    worker_diff_managers: HashMap<String, Arc<RwLock<DifficultyManager>>>,
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
            worker_diff_managers: HashMap::new(),
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
        let worker_login = login.login.clone();
        let account = Self::account_from_login(&login.login);
        let account_worker_len = self.account_workers.get(&account).map_or(0, HashSet::len);
        if account_worker_len >= self.limits.max_workers_per_account {
            return Err(anyhow::anyhow!("too many workers for account"));
        }

        let sub_id = self.next_id();
        let diff_manager = self
            .worker_diff_managers
            .entry(worker_login)
            .or_insert_with(|| Arc::new(RwLock::new(DifficultyManager::new())))
            .clone();
        let worker = MinerWorker::new_with_diff_manager(sub_id, login, diff_manager);
        let worker_id = worker.worker_id;

        let current_job = self
            .current_job
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no job"))?
            .clone();

        let (job, assigned_diff) = Self::get_downstream_job(&worker, true, &current_job);
        self.workers.insert(
            worker_id,
            WorkerSession {
                worker,
                out_tx,
                assigned_diff,
            },
        );
        self.account_workers
            .entry(account.clone())
            .or_default()
            .insert(worker_id);
        self.worker_accounts.insert(worker_id, account);
        Ok((worker_id, job))
    }

    fn clamp_applied_diff(desired_diff: U256, network_diff: U256) -> U256 {
        let mut applied = if desired_diff > network_diff {
            network_diff
        } else {
            desired_diff
        };
        if applied == U256::from(0u64) {
            applied = U256::from(1u64);
        }
        applied
    }

    fn get_downstream_job(
        miner: &MinerWorker,
        set_login: bool,
        upstream_event: &MintBlockEvent,
    ) -> (StratumJobResponse, U256) {
        let login = miner.base_info.clone();
        let desired_diff = miner.diff_manager.read().unwrap().difficulty;
        let network_diff = upstream_event.difficulty;
        let applied_diff = Self::clamp_applied_diff(desired_diff, network_diff);
        let target = difficulty_to_target_hex(applied_diff);
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
            "set downstream job diff:{:?} (worker:{:?}, network:{:?}, applied:{:?})",
            target_hex_to_difficulty(&target).unwrap_or_else(|_| U256::from(1u64)),
            desired_diff,
            network_diff,
            applied_diff
        );

        (
            StratumJobResponse::from(
                upstream_event,
                if set_login { Some(login) } else { None },
                miner.worker_id,
                target,
            ),
            applied_diff,
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
        for (id, session) in self.workers.iter_mut() {
            let _ = session
                .worker
                .diff_manager
                .write()
                .unwrap()
                .maybe_decay(&session.worker.base_info.login);
            let (resp, assigned_diff) = Self::get_downstream_job(&session.worker, false, &event);
            session.assigned_diff = assigned_diff;
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

#[derive(Debug, Serialize)]
struct JsonRpcNotification<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    jsonrpc: Option<&'static str>,
    method: &'static str,
    params: T,
}

#[derive(Clone)]
pub struct App {
    rpc: Arc<dyn NodeRpc>,
    state: Arc<Mutex<GatewayState>>,
    job_poll: Duration,
    pplns: Option<Arc<Mutex<PplnsRuntime>>>,
}

impl App {
    pub fn new(
        rpc: Arc<dyn NodeRpc>,
        limits: StratumLimits,
        job_poll: Duration,
        pplns: Option<Arc<Mutex<PplnsRuntime>>>,
    ) -> Self {
        Self {
            rpc,
            state: Arc::new(Mutex::new(GatewayState::new(limits))),
            job_poll,
            pplns,
        }
    }

    pub async fn settlement_enabled(&self) -> bool {
        let Some(pplns) = self.pplns.as_ref() else {
            return false;
        };
        let pplns = pplns.lock().await;
        pplns.settlement_enabled()
    }

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

    pub async fn register_worker(
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

    pub async fn unregister_worker_hex(&self, worker_id: &str) {
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
                    let err_msg = err.to_string();
                    if err_msg.contains("Mint task is empty Error")
                        || err_msg.contains("Mint task is mismatch Error")
                    {
                        debug!(
                            target: "stratum_server",
                            "forward submit dropped stale task: {}",
                            err_msg
                        );
                    } else {
                        warn!(target: "stratum_server", "forward submit failed: {}", err_msg);
                    }
                }
            }
        });
    }

    pub async fn submit_share(&self, share: ShareRequest) -> SubmitShareResponse {
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
                debug!(
                    target: "stratum_server",
                    "drop late share for missing worker: {}",
                    share.id
                );
                return GatewayState::reject(-1, "worker not found", false);
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

            let (worker_login, assigned_diff) = match state.workers.get(&worker_id) {
                Some(session) => (
                    session.worker.base_info.login.clone(),
                    session.assigned_diff,
                ),
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
                    let _ = state.record_stale_share(worker_id, now);
                    (false, "stale share")
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

            let share_target = match difficult_to_target(assigned_diff) {
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
                let mut diff_manager = session.worker.diff_manager.write().unwrap();
                let _ = diff_manager.try_update(
                    &worker_login,
                    assigned_diff,
                    current_mint_event.difficulty,
                );
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
                difficulty: assigned_diff.as_u64(),
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

    pub async fn run_job_subscribe_loop(self) {
        loop {
            match self.rpc.subscribe_new_mint_blocks().await {
                Ok(mut stream) => loop {
                    match stream.next().await {
                        Some(Ok(job)) => self.on_new_job(job).await,
                        None => {
                            warn!(
                                target: "stratum_server",
                                "upstream mint block stream closed"
                            );
                            break;
                        }
                        Some(Err(err)) => {
                            warn!(
                                target: "stratum_server",
                                "upstream mint block stream failed: {}",
                                err
                            );
                            break;
                        }
                    }
                },
                Err(err) => {
                    warn!(
                        target: "stratum_server",
                        "subscribe_new_mint_blocks failed, fallback to polling: {}",
                        err
                    );
                    match self.rpc.miner_get_job().await {
                        Ok(Some(job)) => self.on_new_job(job).await,
                        Ok(None) => {
                            debug!(target: "stratum_server", "upstream mining.get_job returned none")
                        }
                        Err(poll_err) => {
                            warn!(target: "stratum_server", "upstream mining.get_job failed: {}", poll_err)
                        }
                    }
                }
            }
            sleep(self.job_poll).await;
        }
    }

    pub async fn run_pplns_settlement_loop(self, interval_secs: u64) {
        let interval = Duration::from_secs(interval_secs.max(1));
        loop {
            if let Some(pplns) = &self.pplns {
                let mut pplns = pplns.lock().await;
                if let Err(err) = pplns.settle_tick(self.rpc.as_ref()).await {
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
