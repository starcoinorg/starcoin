use crate::pplns_store::{
    build_pplns_store, CandidateRecord, CandidateStatus, PendingSubmitRecord, PplnsStore,
    ShareRecord,
};
use crate::stratum::{
    AcceptedShareEvent, CandidateBlockEvent, CandidateSolvedEvent, CandidateSubmitEvent,
};
use anyhow::Result;
use futures::executor::block_on;
use starcoin_chain_api::ChainAsyncService;
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{NodeConfig, StratumPplnsConfig};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_types::contract_event::StcContractEvent;
use starcoin_vm2_vm_types::account_config::events::BlockRewardEvent as BlockRewardEventV2;
use starcoin_vm_types::account_config::events::BlockRewardEvent as BlockRewardEventV1;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SHARE_FLUSH_BATCH_SIZE: u64 = 64;

#[derive(Debug)]
struct SettlementTick;

pub struct PplnsService {
    chain_service: ServiceRef<ChainReaderService>,
    config: StratumPplnsConfig,
    store: Box<dyn PplnsStore>,
    dirty_ops: u64,
    local_share_anchor_map: BTreeMap<u64, u64>,
    last_batch_run_millis: Option<u64>,
    last_settled_height: Option<u64>,
}

impl PplnsService {
    fn new(
        chain_service: ServiceRef<ChainReaderService>,
        config: StratumPplnsConfig,
    ) -> Result<Self> {
        let mut store = build_pplns_store(&config)?;
        let last_batch_run_millis = store.last_batch_run_millis()?;
        let last_settled_height = store.last_settled_height()?;
        Ok(Self {
            chain_service,
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

    fn settle_pending(&mut self) -> Result<(u64, u64)> {
        let head = block_on(self.chain_service.main_head_header())?;
        let head_number = head.number();
        let confirmed_head =
            head_number.saturating_sub(self.config.confirmations.saturating_sub(1));
        let mut next_height = self.last_settled_height.unwrap_or(0).saturating_add(1);
        if next_height > confirmed_head {
            return Ok((0, 0));
        }

        let mut settled_count = 0u64;
        let mut orphaned_count = 0u64;
        while next_height <= confirmed_head {
            let Some(main_block) = block_on(self.chain_service.main_block_by_number(next_height))?
            else {
                break;
            };
            let main_block_hash = main_block.id();
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
                        info!(
                            target: "stratum_server",
                            "pplns mark orphan block_hash={}, block_number={}",
                            candidate.block_hash,
                            candidate.block_number
                        );
                    }
                    continue;
                }

                let reward = match reward_for_height {
                    Some(reward) => reward,
                    None => {
                        let reward = self.fetch_block_reward(main_block_hash, next_height)?;
                        reward_for_height = Some(reward);
                        reward
                    }
                };
                let window_shares = self
                    .store
                    .window_shares(candidate.anchor_share_seq, self.config.window_shares)?;
                let credits = Self::allocate_credits(&candidate, &window_shares, reward);
                let now = Self::now_millis();
                if self.store.finalize_confirmed_candidate(
                    &candidate.block_hash,
                    reward,
                    now,
                    credits,
                )? {
                    settled_count = settled_count.saturating_add(1);
                    info!(
                        target: "stratum_server",
                        "pplns settled block_hash={}, block_number={}, reward={}, window_shares={}",
                        candidate.block_hash,
                        candidate.block_number,
                        reward,
                        window_shares.len()
                    );
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

    fn fetch_block_reward(&self, block_hash: HashValue, block_number: u64) -> Result<u128> {
        let txn_infos = block_on(self.chain_service.get_block_txn_infos(block_hash))?;
        let mut reward = 0u128;
        for txn_info in txn_infos {
            let txn_hash = txn_info.transaction_hash();
            let events = block_on(self.chain_service.get_events_by_txn_hash2(txn_hash))?;
            for event_info in events {
                if event_info.block_hash != block_hash {
                    continue;
                }
                if let Some((reward_block_number, amount)) =
                    Self::parse_block_reward_event(&event_info.event)
                {
                    if reward_block_number == block_number {
                        reward = reward.saturating_add(amount);
                    }
                }
            }
        }
        Ok(reward)
    }

    fn parse_block_reward_event(event: &StcContractEvent) -> Option<(u64, u128)> {
        let tag = event.type_tag().to_canonical_string();
        if !tag.contains("BlockRewardEvent") {
            return None;
        }
        match event {
            StcContractEvent::V1(raw) => BlockRewardEventV1::try_from_bytes(raw.event_data())
                .ok()
                .map(|reward| {
                    (
                        reward.block_number,
                        reward.block_reward.saturating_add(reward.gas_fees),
                    )
                }),
            StcContractEvent::V2(raw) => BlockRewardEventV2::try_from_bytes(raw.event_data())
                .ok()
                .map(|reward| {
                    (
                        reward.block_number,
                        reward.block_reward.saturating_add(reward.gas_fees),
                    )
                }),
        }
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

impl ActorService for PplnsService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        if !self.config.enabled {
            info!(target: "stratum_server", "pplns is disabled");
            return Ok(());
        }
        if self.ingest_enabled() {
            ctx.subscribe::<AcceptedShareEvent>();
            ctx.subscribe::<CandidateSubmitEvent>();
            ctx.subscribe::<CandidateSolvedEvent>();
            ctx.subscribe::<CandidateBlockEvent>();
        }
        if self.settlement_enabled() {
            let interval = Duration::from_secs(self.config.settlement_interval_secs);
            ctx.run_interval(interval, |ctx| {
                ctx.notify(SettlementTick);
            });
            ctx.notify(SettlementTick);
        }
        info!(
            target: "stratum_server",
            "pplns started: ingest_enabled={}, settlement_enabled={}, window_shares={}, confirmations={}, interval_secs={}, batch_period_secs={}, last_settled_height={}",
            self.ingest_enabled(),
            self.settlement_enabled(),
            self.config.window_shares,
            self.config.confirmations,
            self.config.settlement_interval_secs,
            self.config.batch_period_secs,
            self.last_settled_height.unwrap_or(0)
        );
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        if self.ingest_enabled() {
            ctx.unsubscribe::<AcceptedShareEvent>();
            ctx.unsubscribe::<CandidateSubmitEvent>();
            ctx.unsubscribe::<CandidateSolvedEvent>();
            ctx.unsubscribe::<CandidateBlockEvent>();
        }
        if self.config.enabled {
            self.flush_store(true);
        }
        Ok(())
    }
}

impl EventHandler<Self, AcceptedShareEvent> for PplnsService {
    fn handle_event(&mut self, event: AcceptedShareEvent, _ctx: &mut ServiceContext<Self>) {
        if !self.ingest_enabled() {
            return;
        }
        let persisted_seq = match self.store.append_share(
            ShareRecord {
                seq: event.seq,
                account: event.account,
                worker_id: event.worker_id,
                difficulty: event.difficulty.max(1),
                accepted_at_millis: event.accepted_at_millis,
            },
            self.config.max_retained_shares,
        ) {
            Ok(seq) => seq,
            Err(err) => {
                warn!(target: "stratum_server", "pplns append share failed: {}", err);
                return;
            }
        };
        self.remember_share_anchor(event.seq, persisted_seq);
        self.dirty_ops = self.dirty_ops.saturating_add(1);
        self.flush_store(false);
    }
}

impl EventHandler<Self, CandidateSubmitEvent> for PplnsService {
    fn handle_event(&mut self, event: CandidateSubmitEvent, _ctx: &mut ServiceContext<Self>) {
        if !self.ingest_enabled() {
            return;
        }
        let anchor_share_seq = self.resolve_anchor_share_seq(event.anchor_share_seq);
        if let Err(err) = self.store.upsert_pending_submit(
            PendingSubmitRecord {
                job_id: event.job_id,
                nonce: event.nonce,
                extra: event.extra,
                account: event.account,
                worker_id: event.worker_id,
                anchor_share_seq,
                expected_block_number: event.expected_block_number,
                submitted_at_millis: event.submitted_at_millis,
            },
            self.config.max_retained_candidates,
        ) {
            warn!(target: "stratum_server", "pplns upsert pending submit failed: {}", err);
            return;
        }
        self.dirty_ops = self.dirty_ops.saturating_add(1);
        self.flush_store(false);
    }
}

impl EventHandler<Self, CandidateSolvedEvent> for PplnsService {
    fn handle_event(&mut self, event: CandidateSolvedEvent, _ctx: &mut ServiceContext<Self>) {
        if !self.ingest_enabled() {
            return;
        }
        let pending = match self
            .store
            .take_pending_submit(&event.job_id, event.nonce, &event.extra)
        {
            Ok(pending) => pending,
            Err(err) => {
                warn!(target: "stratum_server", "pplns take pending submit failed: {}", err);
                return;
            }
        };
        let Some(pending) = pending else {
            debug!(
                target: "stratum_server",
                "pplns solved event missing pending submit: job_id={}, nonce={}, extra={}",
                event.job_id,
                event.nonce,
                event.extra
            );
            return;
        };
        if let Err(err) = self.store.upsert_candidate(
            CandidateRecord {
                block_hash: event.block_hash.to_string(),
                block_number: event.block_number,
                account: pending.account,
                worker_id: pending.worker_id,
                anchor_share_seq: pending.anchor_share_seq,
                found_at_millis: event.found_at_millis,
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
}

impl EventHandler<Self, CandidateBlockEvent> for PplnsService {
    fn handle_event(&mut self, event: CandidateBlockEvent, _ctx: &mut ServiceContext<Self>) {
        if !self.ingest_enabled() {
            return;
        }
        let anchor_share_seq = self.resolve_anchor_share_seq(event.anchor_share_seq);
        if let Err(err) = self.store.upsert_candidate(
            CandidateRecord {
                block_hash: event.block_hash.to_string(),
                block_number: event.block_number,
                account: event.account,
                worker_id: event.worker_id,
                anchor_share_seq,
                found_at_millis: event.found_at_millis,
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
}

impl EventHandler<Self, SettlementTick> for PplnsService {
    fn handle_event(&mut self, _: SettlementTick, _ctx: &mut ServiceContext<Self>) {
        if !self.settlement_enabled() {
            return;
        }
        let now = Self::now_millis();
        if !self.should_run_batch(now) {
            return;
        }
        debug!(
            target: "stratum_server",
            "pplns batch settlement start: at_millis={}, batch_period_secs={}",
            now,
            self.config.batch_period_secs
        );
        let acquired = match self.store.try_acquire_settlement_lock() {
            Ok(acquired) => acquired,
            Err(err) => {
                warn!(
                    target: "stratum_server",
                    "pplns acquire settlement lock failed: {}",
                    err
                );
                return;
            }
        };
        if !acquired {
            debug!(target: "stratum_server", "pplns settlement skipped: lock busy");
            return;
        }
        let settlement_result = self.settle_pending();
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
                        match self.store.balances_len() {
                            Ok(len) => {
                                debug!(
                                    target: "stratum_server",
                                    "pplns balances tracked={}, settled={}, orphaned={}, last_settled_height={}",
                                    len,
                                    settled,
                                    orphaned,
                                    self.last_settled_height.unwrap_or(0)
                                );
                            }
                            Err(err) => {
                                warn!(
                                    target: "stratum_server",
                                    "pplns read balances len failed: {}",
                                    err
                                );
                            }
                        }
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
    }
}

pub struct PplnsServiceFactory;

impl ServiceFactory<PplnsService> for PplnsServiceFactory {
    fn create(ctx: &mut ServiceContext<PplnsService>) -> Result<PplnsService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let chain_service = ctx.service_ref::<ChainReaderService>()?.clone();
        let pplns = config.stratum.pplns();
        PplnsService::new(chain_service, pplns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_credits_weighted() {
        let candidate = CandidateRecord {
            block_hash: "0x1".to_string(),
            block_number: 1,
            account: "finder".to_string(),
            worker_id: "w1".to_string(),
            anchor_share_seq: 10,
            found_at_millis: 0,
            status: CandidateStatus::Pending,
            reward: None,
            settled_at_millis: None,
        };
        let shares = vec![
            ShareRecord {
                seq: 1,
                account: "a".to_string(),
                worker_id: "wa".to_string(),
                difficulty: 1,
                accepted_at_millis: 0,
            },
            ShareRecord {
                seq: 2,
                account: "b".to_string(),
                worker_id: "wb".to_string(),
                difficulty: 3,
                accepted_at_millis: 0,
            },
        ];
        let credits = PplnsService::allocate_credits(&candidate, &shares, 8);
        assert_eq!(credits.get("a"), Some(&2));
        assert_eq!(credits.get("b"), Some(&6));
    }

    #[test]
    fn test_allocate_credits_empty_window_fallback_to_finder() {
        let candidate = CandidateRecord {
            block_hash: "0x1".to_string(),
            block_number: 1,
            account: "finder".to_string(),
            worker_id: "w1".to_string(),
            anchor_share_seq: 10,
            found_at_millis: 0,
            status: CandidateStatus::Pending,
            reward: None,
            settled_at_millis: None,
        };
        let credits = PplnsService::allocate_credits(&candidate, &[], 99);
        assert_eq!(credits.get("finder"), Some(&99));
    }
}
