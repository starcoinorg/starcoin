use crate::pplns_store::{
    build_pplns_store, CandidateRecord, CandidateStatus, PendingSubmitRecord, PplnsStore,
    ShareRecord,
};
use crate::StratumPplnsConfig;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::AsyncRpcClient;
use starcoin_vm2_vm_types::account_config::events::BlockRewardEvent as BlockRewardEventV2;
use starcoin_vm_types::account_config::events::BlockRewardEvent as BlockRewardEventV1;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const SHARE_FLUSH_BATCH_SIZE: u64 = 64;

pub struct PplnsRuntime {
    config: StratumPplnsConfig,
    store: Box<dyn PplnsStore>,
    dirty_ops: u64,
    local_share_anchor_map: BTreeMap<u64, u64>,
    last_batch_run_millis: Option<u64>,
    last_settled_height: Option<u64>,
    integrity_degraded: bool,
}

impl PplnsRuntime {
    pub fn new(config: StratumPplnsConfig) -> Result<Self> {
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
            integrity_degraded: false,
        })
    }

    pub fn config(&self) -> &StratumPplnsConfig {
        &self.config
    }

    pub fn ingest_enabled(&self) -> bool {
        self.config.enabled && self.config.ingest_enabled && !self.integrity_degraded
    }

    pub fn settlement_enabled(&self) -> bool {
        self.config.enabled && self.config.settlement_enabled && !self.integrity_degraded
    }

    fn mark_integrity_degraded(&mut self, stage: &str, err: &dyn std::fmt::Display) {
        if !self.integrity_degraded {
            error!(
                target: "stratum_server",
                "pplns integrity degraded at {}: {}. settlement paused to avoid wrong payout",
                stage,
                err
            );
        } else {
            warn!(
                target: "stratum_server",
                "pplns still degraded at {}: {}",
                stage,
                err
            );
        }
        self.integrity_degraded = true;
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

    pub fn on_accepted_share(
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
                self.mark_integrity_degraded("append_share", &err);
                return;
            }
        };
        self.remember_share_anchor(local_seq, persisted_seq);
        self.dirty_ops = self.dirty_ops.saturating_add(1);
        self.flush_store(false);
    }

    pub fn on_candidate_submit(
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
            self.mark_integrity_degraded("upsert_pending_submit", &err);
            return;
        }
        self.dirty_ops = self.dirty_ops.saturating_add(1);
        self.flush_store(false);
    }

    pub fn on_candidate_solved(
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
                self.mark_integrity_degraded("take_pending_submit", &err);
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
            self.mark_integrity_degraded("upsert_candidate", &err);
            return;
        }
        self.dirty_ops = self.dirty_ops.saturating_add(1);
        self.flush_store(true);
    }

    pub async fn settle_tick(&mut self, rpc: &AsyncRpcClient) -> Result<()> {
        if !self.settlement_enabled() {
            if self.integrity_degraded {
                warn!(
                    target: "stratum_server",
                    "pplns settlement skipped: integrity degraded"
                );
            }
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

    pub fn parse_block_reward_event(tag: &str, data: &[u8]) -> Option<(u64, u128)> {
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

    pub fn allocate_credits(
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
