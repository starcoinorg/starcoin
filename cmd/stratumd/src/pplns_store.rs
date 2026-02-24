use crate::StratumPplnsConfig;
use anyhow::{bail, Context, Result};
use postgres::{Client, NoTls};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const PPLNS_SETTLEMENT_LOCK_ID: i64 = 0x5354_5254_4d50_4c4e;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CandidateStatus {
    Pending,
    Confirmed,
    Orphaned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRecord {
    pub seq: u64,
    pub account: String,
    pub worker_id: String,
    pub difficulty: u64,
    pub accepted_at_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRecord {
    pub block_hash: String,
    pub block_number: u64,
    pub account: String,
    pub worker_id: String,
    pub anchor_share_seq: u64,
    pub found_at_millis: u64,
    pub status: CandidateStatus,
    pub reward: Option<u128>,
    pub settled_at_millis: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSubmitRecord {
    pub job_id: String,
    pub nonce: u32,
    pub extra: String,
    pub account: String,
    pub worker_id: String,
    pub anchor_share_seq: u64,
    pub expected_block_number: u64,
    pub submitted_at_millis: u64,
}

pub trait PplnsStore: Send {
    fn persist(&mut self) -> Result<()>;
    fn append_share(&mut self, share: ShareRecord, max_retained_shares: u64) -> Result<u64>;
    fn upsert_candidate(
        &mut self,
        candidate: CandidateRecord,
        max_retained_candidates: usize,
    ) -> Result<()>;
    fn upsert_pending_submit(
        &mut self,
        pending: PendingSubmitRecord,
        max_retained_candidates: usize,
    ) -> Result<()>;
    fn take_pending_submit(
        &mut self,
        job_id: &str,
        nonce: u32,
        extra: &str,
    ) -> Result<Option<PendingSubmitRecord>>;
    fn window_shares(
        &mut self,
        anchor_share_seq: u64,
        window_shares: u64,
    ) -> Result<Vec<ShareRecord>>;
    fn pending_candidates_by_height(&mut self, block_number: u64) -> Result<Vec<CandidateRecord>>;
    fn mark_candidate_orphaned(&mut self, block_hash: &str, settled_at_millis: u64)
        -> Result<bool>;
    fn finalize_confirmed_candidate(
        &mut self,
        block_hash: &str,
        reward: u128,
        settled_at_millis: u64,
        credits: HashMap<String, u128>,
    ) -> Result<bool>;
    fn remove_confirmed_below(
        &mut self,
        min_block_number: u64,
        max_retained_candidates: usize,
    ) -> Result<()>;
    fn balances_len(&mut self) -> Result<usize>;
    fn try_acquire_settlement_lock(&mut self) -> Result<bool>;
    fn release_settlement_lock(&mut self) -> Result<()>;
    fn last_batch_run_millis(&mut self) -> Result<Option<u64>>;
    fn set_last_batch_run_millis(&mut self, millis: u64) -> Result<()>;
    fn last_settled_height(&mut self) -> Result<Option<u64>>;
    fn set_last_settled_height(&mut self, height: u64) -> Result<()>;
}

pub fn build_pplns_store(config: &StratumPplnsConfig) -> Result<Box<dyn PplnsStore>> {
    let database_url = ensure_database_url(config)?;
    Ok(Box::new(PostgresPplnsStore::connect(database_url)?))
}

fn ensure_database_url(config: &StratumPplnsConfig) -> Result<&str> {
    match config.database_url.as_deref() {
        Some(url) if !url.trim().is_empty() => Ok(url),
        _ => bail!("pplns postgresql backend requires non-empty pplns_database_url"),
    }
}

pub struct PostgresPplnsStore {
    client: Client,
}

impl PostgresPplnsStore {
    fn connect(database_url: &str) -> Result<Self> {
        let mut client = Client::connect(database_url, NoTls)
            .with_context(|| "connect postgresql for pplns store failed")?;
        client.batch_execute(include_str!("../sql/pplns.postgresql.sql"))?;
        Ok(Self { client })
    }

    fn read_meta_u64(&mut self, key: &str) -> Result<Option<u64>> {
        let row = self.client.query_opt(
            "select meta_value from pplns_meta where meta_key = $1",
            &[&key],
        )?;
        row.map(|row| row.get::<_, String>(0))
            .map(|raw| {
                raw.parse::<u64>()
                    .with_context(|| format!("invalid u64 meta value for key {}: {}", key, raw))
            })
            .transpose()
    }

    fn write_meta_u64(&mut self, key: &str, value: u64) -> Result<()> {
        let value = value.to_string();
        self.client.execute(
            "insert into pplns_meta (meta_key, meta_value)
             values ($1, $2)
             on conflict (meta_key) do update set meta_value = excluded.meta_value",
            &[&key, &value],
        )?;
        Ok(())
    }
}

impl PplnsStore for PostgresPplnsStore {
    fn persist(&mut self) -> Result<()> {
        Ok(())
    }

    fn append_share(&mut self, share: ShareRecord, _max_retained_shares: u64) -> Result<u64> {
        let difficulty = to_i64(share.difficulty)?;
        let accepted_at = to_i64(share.accepted_at_millis)?;
        let row = self.client.query_one(
            "insert into pplns_shares (account, worker_id, difficulty, accepted_at_millis)
             values ($1, $2, $3, $4)
             returning id",
            &[&share.account, &share.worker_id, &difficulty, &accepted_at],
        )?;
        to_u64(row.get::<_, i64>(0))
    }

    fn upsert_candidate(
        &mut self,
        candidate: CandidateRecord,
        _max_retained_candidates: usize,
    ) -> Result<()> {
        let block_number = to_i64(candidate.block_number)?;
        let anchor_share_id = to_i64(candidate.anchor_share_seq)?;
        let found_at = to_i64(candidate.found_at_millis)?;
        let settled_at: Option<i64> = match candidate.settled_at_millis {
            Some(v) => Some(to_i64(v)?),
            None => None,
        };
        let reward = candidate.reward.map(|v| v.to_string());
        self.client.execute(
            "insert into pplns_candidates (
                block_hash, block_number, account, worker_id, anchor_share_id, found_at_millis,
                status, reward, settled_at_millis
            ) values ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9)
            on conflict (block_hash) do nothing",
            &[
                &candidate.block_hash,
                &block_number,
                &candidate.account,
                &candidate.worker_id,
                &anchor_share_id,
                &found_at,
                &candidate_status_to_db(&candidate.status),
                &reward,
                &settled_at,
            ],
        )?;
        Ok(())
    }

    fn upsert_pending_submit(
        &mut self,
        pending: PendingSubmitRecord,
        _max_retained_candidates: usize,
    ) -> Result<()> {
        let nonce = i64::from(pending.nonce);
        let anchor_share_id = to_i64(pending.anchor_share_seq)?;
        let expected_block_number = to_i64(pending.expected_block_number)?;
        let submitted_at = to_i64(pending.submitted_at_millis)?;
        self.client.execute(
            "insert into pplns_pending_submits (
                job_id, nonce, extra, account, worker_id, anchor_share_id, expected_block_number, submitted_at_millis
            ) values ($1, $2, $3, $4, $5, $6, $7, $8)
            on conflict (job_id, nonce, extra, worker_id) do nothing",
            &[
                &pending.job_id,
                &nonce,
                &pending.extra,
                &pending.account,
                &pending.worker_id,
                &anchor_share_id,
                &expected_block_number,
                &submitted_at,
            ],
        )?;
        Ok(())
    }

    fn take_pending_submit(
        &mut self,
        job_id: &str,
        nonce: u32,
        extra: &str,
    ) -> Result<Option<PendingSubmitRecord>> {
        let nonce = i64::from(nonce);
        let row = self.client.query_opt(
            "delete from pplns_pending_submits
             where id = (
                select id
                from pplns_pending_submits
                where job_id = $1 and nonce = $2 and extra = $3
                order by id
                limit 1
             )
             returning job_id, nonce, extra, account, worker_id, anchor_share_id,
                       expected_block_number, submitted_at_millis",
            &[&job_id, &nonce, &extra],
        )?;
        row.map(|row| {
            Ok(PendingSubmitRecord {
                job_id: row.get(0),
                nonce: u32::try_from(row.get::<_, i64>(1))
                    .with_context(|| "nonce overflow in pplns_pending_submits")?,
                extra: row.get(2),
                account: row.get(3),
                worker_id: row.get(4),
                anchor_share_seq: to_u64(row.get::<_, i64>(5))?,
                expected_block_number: to_u64(row.get::<_, i64>(6))?,
                submitted_at_millis: to_u64(row.get::<_, i64>(7))?,
            })
        })
        .transpose()
    }

    fn window_shares(
        &mut self,
        anchor_share_seq: u64,
        window_shares: u64,
    ) -> Result<Vec<ShareRecord>> {
        let anchor = to_i64(anchor_share_seq)?;
        let limit = to_i64(window_shares.min(i64::MAX as u64))?;
        let mut rows = self.client.query(
            "select id, account, worker_id, difficulty, accepted_at_millis
             from pplns_shares
             where id <= $1
             order by id desc
             limit $2",
            &[&anchor, &limit],
        )?;
        rows.reverse();
        let mut shares = Vec::with_capacity(rows.len());
        for row in rows {
            shares.push(ShareRecord {
                seq: to_u64(row.get::<_, i64>(0))?,
                account: row.get(1),
                worker_id: row.get(2),
                difficulty: to_u64(row.get::<_, i64>(3))?,
                accepted_at_millis: to_u64(row.get::<_, i64>(4))?,
            });
        }
        Ok(shares)
    }

    fn pending_candidates_by_height(&mut self, block_number: u64) -> Result<Vec<CandidateRecord>> {
        let block_number = to_i64(block_number)?;
        let rows = self.client.query(
            "select block_hash, block_number, account, worker_id, anchor_share_id, found_at_millis
             from pplns_candidates
             where status = 'pending' and block_number = $1
             order by found_at_millis",
            &[&block_number],
        )?;
        let mut candidates = Vec::with_capacity(rows.len());
        for row in rows {
            candidates.push(CandidateRecord {
                block_hash: row.get(0),
                block_number: to_u64(row.get::<_, i64>(1))?,
                account: row.get(2),
                worker_id: row.get(3),
                anchor_share_seq: to_u64(row.get::<_, i64>(4))?,
                found_at_millis: to_u64(row.get::<_, i64>(5))?,
                status: CandidateStatus::Pending,
                reward: None,
                settled_at_millis: None,
            });
        }
        Ok(candidates)
    }

    fn mark_candidate_orphaned(
        &mut self,
        block_hash: &str,
        settled_at_millis: u64,
    ) -> Result<bool> {
        let settled_at = to_i64(settled_at_millis)?;
        let updated = self.client.execute(
            "update pplns_candidates
             set status = 'orphaned', reward = 0, settled_at_millis = $2
             where block_hash = $1 and status = 'pending'",
            &[&block_hash, &settled_at],
        )?;
        Ok(updated > 0)
    }

    fn finalize_confirmed_candidate(
        &mut self,
        block_hash: &str,
        reward: u128,
        settled_at_millis: u64,
        credits: HashMap<String, u128>,
    ) -> Result<bool> {
        let settled_at = to_i64(settled_at_millis)?;
        let reward_str = reward.to_string();
        let created_at = to_i64(settled_at_millis)?;
        let mut tx = self.client.transaction()?;
        let updated = tx.execute(
            "update pplns_candidates
             set status = 'confirmed', reward = $2::numeric, settled_at_millis = $3
             where block_hash = $1 and status = 'pending'",
            &[&block_hash, &reward_str, &settled_at],
        )?;
        if updated == 0 {
            tx.rollback()?;
            return Ok(false);
        }
        for (account, amount) in credits {
            if amount == 0 {
                continue;
            }
            let amount = amount.to_string();
            tx.execute(
                "insert into pplns_ledger_entries (
                    account, block_hash, amount, entry_type, created_at_millis
                ) values ($1, $2, $3::numeric, 'credit', $4)
                on conflict (block_hash, account, entry_type) do nothing",
                &[&account, &block_hash, &amount, &created_at],
            )?;
        }
        tx.commit()?;
        Ok(true)
    }

    fn remove_confirmed_below(
        &mut self,
        min_block_number: u64,
        _max_retained_candidates: usize,
    ) -> Result<()> {
        let min_block_number = to_i64(min_block_number)?;
        self.client.execute(
            "delete from pplns_candidates
             where status <> 'pending' and block_number < $1",
            &[&min_block_number],
        )?;
        Ok(())
    }

    fn balances_len(&mut self) -> Result<usize> {
        let row = self.client.query_one(
            "select count(distinct account)::bigint from pplns_ledger_entries",
            &[],
        )?;
        let len = to_u64(row.get::<_, i64>(0))?;
        usize::try_from(len).with_context(|| "balances length overflow")
    }

    fn try_acquire_settlement_lock(&mut self) -> Result<bool> {
        let row = self.client.query_one(
            "select pg_try_advisory_lock($1)",
            &[&PPLNS_SETTLEMENT_LOCK_ID],
        )?;
        Ok(row.get::<_, bool>(0))
    }

    fn release_settlement_lock(&mut self) -> Result<()> {
        self.client.execute(
            "select pg_advisory_unlock($1)",
            &[&PPLNS_SETTLEMENT_LOCK_ID],
        )?;
        Ok(())
    }

    fn last_batch_run_millis(&mut self) -> Result<Option<u64>> {
        self.read_meta_u64("last_batch_run_millis")
    }

    fn set_last_batch_run_millis(&mut self, millis: u64) -> Result<()> {
        self.write_meta_u64("last_batch_run_millis", millis)
    }

    fn last_settled_height(&mut self) -> Result<Option<u64>> {
        self.read_meta_u64("last_settled_height")
    }

    fn set_last_settled_height(&mut self, height: u64) -> Result<()> {
        self.write_meta_u64("last_settled_height", height)
    }
}

fn candidate_status_to_db(status: &CandidateStatus) -> &'static str {
    match status {
        CandidateStatus::Pending => "pending",
        CandidateStatus::Confirmed => "confirmed",
        CandidateStatus::Orphaned => "orphaned",
    }
}

fn to_i64(value: u64) -> Result<i64> {
    i64::try_from(value).with_context(|| format!("value {} does not fit into i64", value))
}

fn to_u64(value: i64) -> Result<u64> {
    u64::try_from(value).with_context(|| format!("value {} does not fit into u64", value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_backend_requires_database_url() {
        let config = StratumPplnsConfig {
            enabled: true,
            ingest_enabled: true,
            settlement_enabled: true,
            window_shares: 1,
            confirmations: 1,
            settlement_interval_secs: 1,
            batch_period_secs: 60,
            max_retained_shares: 1,
            max_retained_candidates: 64,
            database_url: None,
        };
        let err = match build_pplns_store(&config) {
            Ok(_) => panic!("missing database url should fail"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("pplns_database_url"),
            "unexpected error: {err}"
        );
    }
}
