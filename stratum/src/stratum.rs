use crate::{rpc::*, target_hex_to_difficulty};
use anyhow::Result;
use futures::channel::mpsc;
use starcoin_config::StratumLimits;
use starcoin_consensus::{difficult_to_target, Consensus};
use starcoin_logger::prelude::*;
use starcoin_miner::{
    MinerService, SubmitSealRequest as MinerSubmitSealRequest, UpdateSubscriberNumRequest,
};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
};
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::U256;
use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::sync::atomic;
use std::sync::Arc;
use std::time::{Duration, Instant};

const ERROR_WINDOW_SECS: u64 = 300;
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
    event: MintBlockEvent,
    ts: Instant,
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
    account_workers: HashMap<String, HashSet<WorkerId>>,
    worker_accounts: HashMap<WorkerId, String>,
    stats: ShareStats,
    last_stats_log: Instant,
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
            account_workers: Default::default(),
            worker_accounts: Default::default(),
            stats: ShareStats::default(),
            last_stats_log: Instant::now(),
            limits,
        }
    }

    fn next_id(&self) -> u32 {
        self.uid.fetch_add(1, atomic::Ordering::SeqCst)
    }

    fn sync_upstream_job(&mut self) -> Result<Option<MintBlockEvent>> {
        let service = self.miner_service.clone();
        let subscribers_num = self.mint_block_subscribers.len() as u32;
        futures::executor::block_on(service.send(UpdateSubscriberNumRequest {
            number: Some(subscribers_num),
        }))
    }

    fn get_downstream_job(
        miner: &MinerWorker,
        set_login: bool,
        upstreaum_event: &MintBlockEvent,
    ) -> StratumJobResponse {
        let login = miner.base_info.clone();

        let target = miner.diff_manager.read().unwrap().get_target();
        if target.len() != TARGET_HEX_LEN || !Self::is_hex(&target) {
            warn!(
                target: "stratum",
                "unexpected target hex len:{} target:{}",
                target.len(),
                target
            );
        }
        info!(
            "set downstream job diff:{:?}",
            target_hex_to_difficulty(&target).unwrap()
        );
        StratumJobResponse::from(
            upstreaum_event,
            if set_login { Some(login) } else { None },
            miner.worker_id,
            target,
        )
    }

    fn dispatch_job_to_clients(&mut self, event: MintBlockEvent) {
        self.cache_job(&event);
        let mut remove_outdated = vec![];
        for (id, (ch, worker)) in self.mint_block_subscribers.iter() {
            let _ = worker
                .diff_manager
                .write()
                .unwrap()
                .maybe_decay(&worker.base_info.login);
            let job = Self::get_downstream_job(worker, false, &event);
            info!(target: "stratum", "dispatch startum job:{:?}", job);
            let mut ch = ch.clone();
            if let Err(err) = ch.try_send(job) {
                if err.is_disconnected() {
                    warn!("stratum disconnect worker:{:?}", err);
                    remove_outdated.push(*id);
                } else if err.is_full() {
                    warn!(
                        target: "stratum",
                        "subscription {:?} channel full, drop worker",
                        id
                    );
                    remove_outdated.push(*id);
                }
            }
        }
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
        self.recent_jobs.insert(
            job_id,
            JobCacheEntry {
                event: event.clone(),
                ts: now,
            },
        );
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
    }

    fn find_job_event(&mut self, job_id: [u8; 8], now: Instant) -> Option<MintBlockEvent> {
        self.recent_jobs.retain(|_, entry| {
            now.duration_since(entry.ts) <= Duration::from_secs(self.limits.stale_window_secs)
        });
        self.recent_jobs
            .get(&job_id)
            .map(|entry| entry.event.clone())
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

    fn is_hex(s: &str) -> bool {
        s.bytes().all(|b| b.is_ascii_hexdigit())
    }

    fn log_disconnect(&self, worker_id: WorkerId, reason: &str) {
        warn!(
            target: "stratum",
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
                target: "stratum",
                "share_stats accepted:{} invalid:{} duplicate:{} stale:{} job_miss:{} rate_limited:{}",
                self.stats.accepted,
                self.stats.invalid,
                self.stats.duplicate,
                self.stats.stale,
                self.stats.job_miss,
                self.stats.rate_limited
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
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<MintBlockEvent>();
        Ok(())
    }
}

impl EventHandler<Self, MintBlockEvent> for Stratum {
    fn handle_event(&mut self, event: MintBlockEvent, _ctx: &mut ServiceContext<Stratum>) {
        self.dispatch_job_to_clients(event);
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
        info!(target: "stratum", "receive subscribe event {:?},sub_id:{}", login, sub_id);
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
        let event = self.sync_upstream_job()?;
        if let Some(ref e) = event {
            self.cache_job(e);
        }
        let downstream_job = event.as_ref().and_then(|event| {
            self.mint_block_subscribers
                .get(&worker_id)
                .map(|(_, worker)| Self::get_downstream_job(worker, true, event))
        });
        if let Some(downstream_job) = downstream_job {
            info!(target:"stratum", "Respond to stratum subscribe:{:?}", downstream_job);
            if let Err(err) = sender.try_send(downstream_job) {
                error!(target: "stratum", "Failed to send MintBlockEvent: {}", err);
                self.drop_worker_state(worker_id);
                return Err(anyhow::anyhow!("subscribe job channel unavailable"));
            }
        } else {
            warn!(target: "stratum", "current mint job is empty");
        }
        Ok(receiver)
    }
}

impl ServiceHandler<Self, SubmitShareEvent> for Stratum {
    fn handle(
        &mut self,
        msg: SubmitShareEvent,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<SubmitShareResponse> {
        let share = msg.0;
        info!(target: "stratum", "received submit share event:{:?}", &share);
        let now = Instant::now();

        let worker_id = match WorkerId::from_hex(share.id.clone()) {
            Ok(worker_id) => worker_id,
            Err(err) => {
                warn!(target: "stratum", "invalid worker id: {}", err);
                self.stats.invalid = self.stats.invalid.saturating_add(1);
                self.note_stat(now);
                return Ok(Self::reject(-1, "invalid worker id", false));
            }
        };
        if !self.mint_block_subscribers.contains_key(&worker_id) {
            self.stats.job_miss = self.stats.job_miss.saturating_add(1);
            self.note_stat(now);
            return Ok(Self::reject(-1, "worker not found", false));
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

        let current_mint_event = match self.sync_upstream_job()? {
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
                warn!(target: "stratum", "invalid job id: {}", err);
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

        let submit_job_id = JobId::from_bob(&current_mint_event.minting_blob);
        if job_id != submit_job_id {
            if self.find_job_event(job_id.job_id, now).is_some() {
                self.stats.stale = self.stats.stale.saturating_add(1);
                self.note_stat(now);
                let disconnect = self.record_stale_share(worker_id, now);
                if disconnect {
                    self.log_disconnect(worker_id, "stale share");
                    self.drop_worker_state(worker_id);
                }
                return Ok(Self::reject(-1, "stale share", disconnect));
            }
            warn!(
                target: "stratum",
                "received job mismatch with current job,{:?},{:?}",
                job_id,
                submit_job_id
            );
            let disconnect = self.record_job_miss(worker_id, now);
            self.stats.job_miss = self.stats.job_miss.saturating_add(1);
            self.note_stat(now);
            if disconnect {
                self.log_disconnect(worker_id, "job not found");
                self.drop_worker_state(worker_id);
            }
            return Ok(Self::reject(-1, "job not found", disconnect));
        }

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

        let seal: MinerSubmitSealRequest = match share.clone().try_into() {
            Ok(seal) => seal,
            Err(err) => {
                warn!(target: "stratum", "invalid share: {}", err);
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

        let mining_blob = Self::build_mining_blob(&current_mint_event, &seal.extra)?;
        let pow_hash = current_mint_event.strategy.calculate_pow_hash(
            &mining_blob,
            seal.nonce,
            &seal.extra,
        )?;
        let pow_hash_u256: U256 = pow_hash.into();
        let difficulty = diff_manager.read().unwrap().difficulty;
        let share_target = difficult_to_target(difficulty)?;
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
        if job_id == submit_job_id && pow_hash_u256 <= network_target {
            if let Err(err) = current_mint_event.strategy.verify_blob(
                current_mint_event.minting_blob.clone(),
                seal.nonce,
                seal.extra,
                current_mint_event.difficulty,
            ) {
                warn!(target: "stratum", "verify blob failed: {}", err);
            } else {
                let mut forward_seal = seal;
                forward_seal.minting_blob = current_mint_event.minting_blob;
                self.miner_service.try_send(forward_seal)?;
            }
        }

        self.stats.accepted = self.stats.accepted.saturating_add(1);
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
