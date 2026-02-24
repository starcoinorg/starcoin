use crate::{rpc::*, target_hex_to_difficulty};
use anyhow::Result;
use futures::channel::mpsc;
use starcoin_config::StratumLimits;
use starcoin_consensus::{difficult_to_target, Consensus};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_miner::{
    MinerService, SubmitSealRequest as MinerSubmitSealRequest, UpdateSubscriberNumRequest,
};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
};
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::system_events::{GenerateBlockEvent, MinedBlock, MintBlockEvent};
use starcoin_types::U256;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::sync::atomic;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const ERROR_WINDOW_SECS: u64 = 120;
const MAX_RECENT_JOBS: usize = 512;
const STATS_LOG_INTERVAL_SECS: u64 = 60;
const MAX_NONCE_HEX_LEN: usize = 8;
const JOB_ID_HEX_LEN: usize = 16;
const WORKER_ID_HEX_LEN: usize = 8;
const TARGET_HEX_LEN: usize = 16;
const JOB_CHANNEL_CAP: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ShareKey {
    job_id: [u8; 8],
    nonce: u32,
    extra: [u8; 4],
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
struct CandidateSubmitKey {
    job_id: [u8; 8],
    nonce: u32,
    extra: [u8; 4],
}

#[derive(Debug, Clone)]
struct CandidateMeta {
    worker_id: WorkerId,
    account: String,
    share_seq: u64,
    submitted_at: Instant,
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

#[derive(Debug, Clone)]
pub struct AcceptedShareEvent {
    pub seq: u64,
    pub worker_id: String,
    pub account: String,
    pub difficulty: u64,
    pub accepted_at_millis: u64,
}

#[derive(Debug, Clone)]
pub struct CandidateBlockEvent {
    pub block_hash: HashValue,
    pub block_number: u64,
    pub worker_id: String,
    pub account: String,
    pub anchor_share_seq: u64,
    pub found_at_millis: u64,
}

#[derive(Debug, Clone)]
pub struct CandidateSubmitEvent {
    pub job_id: String,
    pub nonce: u32,
    pub extra: String,
    pub worker_id: String,
    pub account: String,
    pub anchor_share_seq: u64,
    pub expected_block_number: u64,
    pub submitted_at_millis: u64,
}

#[derive(Debug, Clone)]
pub struct CandidateSolvedEvent {
    pub job_id: String,
    pub nonce: u32,
    pub extra: String,
    pub block_hash: HashValue,
    pub block_number: u64,
    pub found_at_millis: u64,
}

pub struct Stratum {
    uid: atomic::AtomicU32,
    mint_block_subscribers: HashMap<WorkerId, (mpsc::Sender<StratumJobResponse>, MinerWorker)>,
    miner_service: ServiceRef<MinerService>,
    limits: StratumLimits,
    share_dedup: HashMap<WorkerId, HashMap<ShareKey, Instant>>,
    invalid_share_counters: HashMap<WorkerId, ErrorCounter>,
    job_miss_counters: HashMap<WorkerId, ErrorCounter>,
    stale_share_counters: HashMap<WorkerId, ErrorCounter>,
    share_rate: HashMap<WorkerId, RateCounter>,
    recent_jobs: HashMap<[u8; 8], JobCacheEntry>,
    recent_job_queue: VecDeque<[u8; 8]>,
    solved_jobs: HashMap<[u8; 8], Instant>,
    pending_candidates: HashMap<CandidateSubmitKey, CandidateMeta>,
    current_job: Option<MintBlockEvent>,
    account_workers: HashMap<String, HashSet<WorkerId>>,
    worker_accounts: HashMap<WorkerId, String>,
    stats: ShareStats,
    last_stats_log: Instant,
    forwarded_blocks: u64,
    accepted_share_seq: u64,
}

impl Stratum {
    fn new(miner_service: ServiceRef<MinerService>, limits: StratumLimits) -> Self {
        Self {
            miner_service,
            uid: atomic::AtomicU32::new(1),
            mint_block_subscribers: Default::default(),
            share_dedup: Default::default(),
            invalid_share_counters: Default::default(),
            job_miss_counters: Default::default(),
            stale_share_counters: Default::default(),
            share_rate: Default::default(),
            recent_jobs: Default::default(),
            recent_job_queue: Default::default(),
            solved_jobs: Default::default(),
            pending_candidates: Default::default(),
            current_job: None,
            account_workers: Default::default(),
            worker_accounts: Default::default(),
            stats: ShareStats::default(),
            last_stats_log: Instant::now(),
            forwarded_blocks: 0,
            accepted_share_seq: 0,
            limits,
        }
    }

    fn next_id(&self) -> u32 {
        self.uid.fetch_add(1, atomic::Ordering::SeqCst)
    }

    fn sync_upstream_job(&mut self) -> Result<Option<MintBlockEvent>> {
        let service = self.miner_service.clone();
        let subscribers_num = self.mint_block_subscribers.len() as u32;
        let latest = futures::executor::block_on(service.send(UpdateSubscriberNumRequest {
            number: Some(subscribers_num),
        }))?;
        if latest.is_none() && subscribers_num > 0 {
            if let Err(err) = service.notify(GenerateBlockEvent::default()) {
                warn!(
                    target: "stratum_server",
                    "notify generate block event failed: {}",
                    err
                );
            }
        }
        Ok(latest)
    }

    fn get_downstream_job(
        miner: &MinerWorker,
        set_login: bool,
        upstreaum_event: &MintBlockEvent,
    ) -> StratumJobResponse {
        let login = miner.base_info.clone();

        let desired_diff = miner.diff_manager.read().unwrap().difficulty;
        let network_diff = upstreaum_event.difficulty;
        let target = crate::difficulty_to_target_hex(desired_diff);
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
            target_hex_to_difficulty(&target).unwrap(),
            desired_diff,
            network_diff
        );
        StratumJobResponse::from(
            upstreaum_event,
            if set_login { Some(login) } else { None },
            miner.worker_id,
            target,
        )
    }

    fn dispatch_job_to_clients(&mut self, event: MintBlockEvent) {
        self.current_job = Some(event.clone());
        self.cache_job(&event);
        let mut remove_outdated = vec![];
        for (id, (ch, worker)) in self.mint_block_subscribers.iter() {
            let _ = worker
                .diff_manager
                .write()
                .unwrap()
                .maybe_decay(&worker.base_info.login);
            let job = Self::get_downstream_job(worker, false, &event);
            info!(target: "stratum_server", "dispatch startum job:{:?}", job);
            let mut ch = ch.clone();
            if let Err(err) = ch.try_send(job) {
                if err.is_disconnected() {
                    warn!(target: "stratum_server", "stratum disconnect worker:{:?}", err);
                    remove_outdated.push(*id);
                } else if err.is_full() {
                    warn!(
                        target: "stratum_server",
                        "subscription {:?} channel full, drop worker",
                        id
                    );
                    remove_outdated.push(*id);
                }
            }
        }
        let need_sync = !remove_outdated.is_empty();
        for id in remove_outdated {
            self.mint_block_subscribers.remove(&id);
            self.share_dedup.remove(&id);
            self.invalid_share_counters.remove(&id);
            self.job_miss_counters.remove(&id);
            self.stale_share_counters.remove(&id);
            self.share_rate.remove(&id);
            if let Some(account) = self.worker_accounts.remove(&id) {
                if let Some(set) = self.account_workers.get_mut(&account) {
                    set.remove(&id);
                    if set.is_empty() {
                        self.account_workers.remove(&account);
                    }
                }
            }
        }
        if need_sync {
            if let Err(err) = self.sync_upstream_job() {
                warn!(target: "stratum_server", "sync upstream job failed: {}", err);
            }
        }
    }

    fn refresh_upstream_job_if_changed(&mut self) {
        match self.sync_upstream_job() {
            Ok(Some(event)) => {
                let changed = self.current_job.as_ref().is_none_or(|current| {
                    JobId::from_bob(&current.minting_blob) != JobId::from_bob(&event.minting_blob)
                });
                if changed {
                    let refreshed_job_id = JobId::from_bob(&event.minting_blob).encode();
                    self.current_job = Some(event.clone());
                    self.cache_job(&event);
                    debug!(
                        target: "stratum_server",
                        "refresh current job from upstream without dispatch, job_id={}",
                        refreshed_job_id
                    );
                }
            }
            Ok(None) => {}
            Err(err) => {
                warn!(
                    target: "stratum_server",
                    "sync upstream job failed during submit: {}",
                    err
                );
            }
        }
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

    fn drop_worker_state(&mut self, worker_id: WorkerId) {
        self.mint_block_subscribers.remove(&worker_id);
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

    fn build_mining_blob(event: &MintBlockEvent, extra: &BlockHeaderExtra) -> Result<Vec<u8>> {
        let mut blob = event.minting_blob.clone();
        if blob.len() < 39 {
            return Err(anyhow::anyhow!("invalid minting blob"));
        }
        blob[35..39].copy_from_slice(extra.as_slice());
        Ok(blob)
    }

    fn cache_job(&mut self, event: &MintBlockEvent) {
        let job_id = JobId::from_bob(&event.minting_blob).job_id;
        let now = Instant::now();
        self.retain_pending_candidates(now);
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

    fn next_share_seq(&mut self) -> u64 {
        self.accepted_share_seq = self.accepted_share_seq.saturating_add(1);
        self.accepted_share_seq
    }

    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis() as u64)
    }

    fn retain_pending_candidates(&mut self, now: Instant) {
        self.pending_candidates.retain(|_, meta| {
            now.duration_since(meta.submitted_at)
                <= Duration::from_secs(self.limits.stale_window_secs)
        });
    }

    fn is_hex(s: &str) -> bool {
        s.bytes().all(|b| b.is_ascii_hexdigit())
    }

    fn is_solved_job(&mut self, job_id: [u8; 8], now: Instant) -> bool {
        self.solved_jobs.retain(|_, ts| {
            now.duration_since(*ts) <= Duration::from_secs(self.limits.stale_window_secs)
        });
        self.solved_jobs.contains_key(&job_id)
    }

    fn log_disconnect(&self, worker_id: WorkerId, reason: &str) {
        warn!(
            target: "stratum_server",
            "disconnect worker {}: {}",
            worker_id.to_hex(),
            reason
        );
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

    fn account_from_login(login: &str) -> String {
        login.split('.').next().unwrap_or(login).to_string()
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
}

impl ActorService for Stratum {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.set_mailbox_capacity(1024);
        ctx.subscribe::<MintBlockEvent>();
        ctx.subscribe::<MinedBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MintBlockEvent>();
        ctx.unsubscribe::<MinedBlock>();
        Ok(())
    }
}

impl EventHandler<Self, MintBlockEvent> for Stratum {
    fn handle_event(&mut self, event: MintBlockEvent, _ctx: &mut ServiceContext<Stratum>) {
        self.dispatch_job_to_clients(event);
    }
}

impl EventHandler<Self, MinedBlock> for Stratum {
    fn handle_event(&mut self, event: MinedBlock, ctx: &mut ServiceContext<Stratum>) {
        let header = event.0.header();
        let blob = header.as_pow_header_blob();
        let job_id = JobId::from_bob(&blob).job_id;
        let now = Instant::now();
        self.retain_pending_candidates(now);
        self.solved_jobs.insert(job_id, now);
        let candidate_key = CandidateSubmitKey {
            job_id,
            nonce: header.nonce(),
            extra: *header.extra().as_slice(),
        };
        ctx.broadcast(CandidateSolvedEvent {
            job_id: hex::encode(job_id),
            nonce: candidate_key.nonce,
            extra: hex::encode(candidate_key.extra),
            block_hash: header.id(),
            block_number: header.number(),
            found_at_millis: header.timestamp(),
        });
        if let Some(candidate) = self.pending_candidates.remove(&candidate_key) {
            ctx.broadcast(CandidateBlockEvent {
                block_hash: header.id(),
                block_number: header.number(),
                worker_id: candidate.worker_id.to_hex(),
                account: candidate.account,
                anchor_share_seq: candidate.share_seq,
                found_at_millis: header.timestamp(),
            });
        } else {
            debug!(
                target: "stratum_server",
                "missing candidate mapping for solved block: job_id={}, nonce={}, extra={}",
                hex::encode(job_id),
                header.nonce(),
                hex::encode(header.extra().as_slice())
            );
        }
        debug!(
            target: "stratum_server",
            "mark job {} as solved",
            hex::encode(job_id)
        );
    }
}

impl ServiceHandler<Self, SubscribeJobEvent> for Stratum {
    fn handle(
        &mut self,
        msg: SubscribeJobEvent,
        _ctx: &mut ServiceContext<Self>,
    ) -> anyhow::Result<mpsc::Receiver<StratumJobResponse>> {
        let SubscribeJobEvent(login) = msg;
        let (mut sender, receiver) = mpsc::channel(JOB_CHANNEL_CAP);
        let sub_id = self.next_id();
        info!(target: "stratum_server", "receive subscribe event {:?},sub_id:{}", login, sub_id);
        let account = Self::account_from_login(&login.login);
        let entry = self.account_workers.entry(account.clone()).or_default();
        if entry.len() >= self.limits.max_workers_per_account {
            return Err(anyhow::anyhow!("too many workers for account"));
        }
        let miner_worker = MinerWorker::new(sub_id, login);
        let worker_id = miner_worker.worker_id;
        self.mint_block_subscribers
            .insert(worker_id, (sender.clone(), miner_worker));
        entry.insert(worker_id);
        self.worker_accounts.insert(worker_id, account);
        match self.sync_upstream_job() {
            Ok(Some(event)) => {
                self.current_job = Some(event.clone());
                self.cache_job(&event);
            }
            Ok(None) => {}
            Err(err) => {
                warn!(target: "stratum_server", "sync upstream job failed: {}", err);
            }
        }
        let event = self.current_job.clone();
        let downstream_job = event.as_ref().and_then(|event| {
            self.mint_block_subscribers
                .get(&worker_id)
                .map(|(_, worker)| Self::get_downstream_job(worker, true, event))
        });
        if let Some(downstream_job) = downstream_job {
            info!(
                target: "stratum_server",
                "Respond to stratum subscribe:{:?}",
                downstream_job
            );
            if let Err(err) = sender.try_send(downstream_job) {
                error!(target: "stratum_server", "Failed to send MintBlockEvent: {}", err);
                self.drop_worker_state(worker_id);
                return Err(anyhow::anyhow!("subscribe job channel unavailable"));
            }
        } else {
            warn!(target: "stratum_server", "current mint job is empty");
        }
        Ok(receiver)
    }
}

impl ServiceHandler<Self, SubmitShareEvent> for Stratum {
    fn handle(
        &mut self,
        msg: SubmitShareEvent,
        ctx: &mut ServiceContext<Self>,
    ) -> Result<SubmitShareResponse> {
        // Keep current job fresh under real miner traffic; otherwise solved-job shares can
        // pile up against an outdated template and never reach MinerService.
        self.refresh_upstream_job_if_changed();

        let share = msg.0;
        info!(target: "stratum_server", "received submit share event:{:?}", &share);
        let now = Instant::now();
        self.retain_pending_candidates(now);

        let worker_id = match WorkerId::from_hex(share.id.clone()) {
            Ok(worker_id) => worker_id,
            Err(err) => {
                warn!(target: "stratum_server", "invalid worker id: {}", err);
                self.stats.invalid = self.stats.invalid.saturating_add(1);
                self.note_stat(now);
                return Ok(Self::reject(-1, "invalid worker id", false));
            }
        };
        if !self.mint_block_subscribers.contains_key(&worker_id) {
            self.stats.job_miss = self.stats.job_miss.saturating_add(1);
            self.note_stat(now);
            self.log_disconnect(worker_id, "worker not found");
            self.drop_worker_state(worker_id);
            return Ok(Self::reject(-1, "worker not found", true));
        }

        let validation = self.validate_share_params(worker_id, &share, now);
        if !matches!(validation, SubmitShareResponse::Accepted) {
            self.stats.invalid = self.stats.invalid.saturating_add(1);
            self.note_stat(now);
            return Ok(validation);
        }

        if self.check_rate_limit(worker_id, now) {
            self.stats.rate_limited = self.stats.rate_limited.saturating_add(1);
            self.note_stat(now);
            return Ok(Self::reject(-1, "rate limited", false));
        }

        let current_mint_event = match self.current_job.clone() {
            Some(event) => event,
            None => {
                let disconnect = self.record_job_miss(worker_id, now);
                self.stats.job_miss = self.stats.job_miss.saturating_add(1);
                self.note_stat(now);
                if disconnect {
                    self.log_disconnect(worker_id, "job not found");
                    self.drop_worker_state(worker_id);
                }
                return Ok(Self::reject(-1, "job not found", disconnect));
            }
        };

        let job_id = match JobId::new(&share.job_id) {
            Ok(job_id) => job_id,
            Err(err) => {
                warn!(target: "stratum_server", "invalid job id: {}", err);
                let disconnect = self.record_invalid_share(worker_id, now);
                self.stats.invalid = self.stats.invalid.saturating_add(1);
                self.note_stat(now);
                if disconnect {
                    self.log_disconnect(worker_id, "invalid job id");
                    self.drop_worker_state(worker_id);
                }
                return Ok(Self::reject(-1, "invalid job id", disconnect));
            }
        };

        let (diff_manager, worker_login) = match self.mint_block_subscribers.get(&worker_id) {
            Some((_job_sender, worker)) => (worker.diff_manager(), worker.base_info.login.clone()),
            None => {
                let disconnect = self.record_job_miss(worker_id, now);
                if disconnect {
                    self.log_disconnect(worker_id, "worker not found");
                    self.drop_worker_state(worker_id);
                }
                return Ok(Self::reject(-1, "worker not found", disconnect));
            }
        };
        let worker_account = Self::account_from_login(&worker_login);

        let seal: MinerSubmitSealRequest = match share.clone().try_into() {
            Ok(seal) => seal,
            Err(err) => {
                warn!(target: "stratum_server", "invalid share: {}", err);
                let disconnect = self.record_invalid_share(worker_id, now);
                self.stats.invalid = self.stats.invalid.saturating_add(1);
                self.note_stat(now);
                if disconnect {
                    self.log_disconnect(worker_id, "invalid share");
                    self.drop_worker_state(worker_id);
                }
                return Ok(Self::reject(-1, "invalid share", disconnect));
            }
        };

        let share_key = ShareKey {
            job_id: job_id.job_id,
            nonce: seal.nonce,
            extra: *seal.extra.as_slice(),
        };
        if self.is_duplicate_share(worker_id, share_key, now) {
            let disconnect = self.record_invalid_share(worker_id, now);
            self.stats.duplicate = self.stats.duplicate.saturating_add(1);
            self.note_stat(now);
            if disconnect {
                self.log_disconnect(worker_id, "duplicate share");
                self.drop_worker_state(worker_id);
            }
            return Ok(Self::reject(-1, "duplicate share", disconnect));
        }

        let submit_job_id = JobId::from_bob(&current_mint_event.minting_blob);
        if job_id != submit_job_id {
            let job_known = self.recent_jobs.contains_key(&job_id.job_id);
            let (disconnect, reason) = if job_known {
                self.stats.stale = self.stats.stale.saturating_add(1);
                (self.record_stale_share(worker_id, now), "stale share")
            } else {
                self.stats.job_miss = self.stats.job_miss.saturating_add(1);
                (self.record_job_miss(worker_id, now), "job not found")
            };
            debug!(
                target: "stratum_server",
                "received job mismatch with current job, submitted={:?}, current={:?}, worker={}, known_job={}",
                job_id,
                submit_job_id,
                share.id,
                job_known
            );
            self.note_stat(now);
            if disconnect {
                self.log_disconnect(worker_id, reason);
                self.drop_worker_state(worker_id);
            }
            return Ok(Self::reject(-1, reason, disconnect));
        }

        let mining_blob = Self::build_mining_blob(&current_mint_event, &seal.extra)?;
        let pow_hash = current_mint_event.strategy.calculate_pow_hash(
            &mining_blob,
            seal.nonce,
            &seal.extra,
        )?;
        let pow_hash_u256: U256 = pow_hash.into();
        let desired = diff_manager.read().unwrap().difficulty;
        let network = current_mint_event.difficulty;
        let effective = if desired > network { network } else { desired };
        let share_target = difficult_to_target(effective)?;
        if pow_hash_u256 > share_target {
            let disconnect = self.record_invalid_share(worker_id, now);
            self.stats.invalid = self.stats.invalid.saturating_add(1);
            self.note_stat(now);
            if disconnect {
                self.log_disconnect(worker_id, "low difficulty share");
                self.drop_worker_state(worker_id);
            }
            return Ok(Self::reject(-1, "low difficulty share", disconnect));
        }

        let _updated_diff = diff_manager.write().unwrap().try_update(worker_login);

        let network_target = difficult_to_target(current_mint_event.difficulty)?;
        let solved = self.is_solved_job(job_id.job_id, now);
        let mut forwarded_candidate_key: Option<CandidateSubmitKey> = None;
        if job_id == submit_job_id && pow_hash_u256 <= network_target {
            if solved {
                debug!(
                    target: "stratum_server",
                    "skip forwarding solved job {}",
                    share.job_id
                );
            } else {
                let mut forward_seal = seal;
                forward_seal.minting_blob = current_mint_event.minting_blob;
                debug!(
                    target: "stratum_server",
                    "forward block candidate job_id={}, worker={}, nonce={}",
                    share.job_id,
                    share.id,
                    share.nonce
                );
                self.miner_service.try_send(forward_seal)?;
                self.forwarded_blocks = self.forwarded_blocks.saturating_add(1);
                forwarded_candidate_key = Some(CandidateSubmitKey {
                    job_id: job_id.job_id,
                    nonce: share_key.nonce,
                    extra: share_key.extra,
                });
            }
        }

        let share_seq = self.next_share_seq();
        self.stats.accepted = self.stats.accepted.saturating_add(1);
        ctx.broadcast(AcceptedShareEvent {
            seq: share_seq,
            worker_id: worker_id.to_hex(),
            account: worker_account.clone(),
            difficulty: effective.as_u64(),
            accepted_at_millis: Self::now_millis(),
        });
        if let Some(candidate_key) = forwarded_candidate_key {
            let submitted_at_millis = Self::now_millis();
            ctx.broadcast(CandidateSubmitEvent {
                job_id: hex::encode(candidate_key.job_id),
                nonce: candidate_key.nonce,
                extra: hex::encode(candidate_key.extra),
                worker_id: worker_id.to_hex(),
                account: worker_account.clone(),
                anchor_share_seq: share_seq,
                expected_block_number: current_mint_event.block_number,
                submitted_at_millis,
            });
            self.pending_candidates.insert(
                candidate_key,
                CandidateMeta {
                    worker_id,
                    account: worker_account,
                    share_seq,
                    submitted_at: now,
                },
            );
        }
        self.note_stat(now);
        Ok(Self::accept())
    }
}

impl ServiceHandler<Self, UnsubscribeWorkerEvent> for Stratum {
    fn handle(
        &mut self,
        msg: UnsubscribeWorkerEvent,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<()> {
        self.drop_worker_state(msg.worker_id);
        if let Err(err) = self.sync_upstream_job() {
            warn!(target: "stratum_server", "sync upstream job failed: {}", err);
        }
        Ok(())
    }
}

pub struct StratumFactory;

impl ServiceFactory<Stratum> for StratumFactory {
    fn create(ctx: &mut ServiceContext<Stratum>) -> Result<Stratum> {
        let miner_service = ctx.service_ref::<MinerService>()?.clone();
        let config = ctx.get_shared::<Arc<starcoin_config::NodeConfig>>()?;
        let limits = config.stratum.limits();
        Ok(Stratum::new(miner_service, limits))
    }
}
