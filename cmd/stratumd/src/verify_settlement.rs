use anyhow::Result;
use clap::Args;
use postgres::{Client, NoTls};
use starcoin_crypto::HashValue;
use starcoin_stratumd::node_rpc::{build_sync_rpc_client, NodeRpcSync};
use starcoin_stratumd::pplns::PplnsRuntime;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct VerifySettlementOpt {
    #[arg(long, default_value = "ws://127.0.0.1:9870")]
    pub node_rpc: String,

    #[arg(long)]
    pub pplns_database_url: String,

    #[arg(long)]
    pub from_height: Option<u64>,

    #[arg(long)]
    pub to_height: Option<u64>,
}

#[derive(Debug, Clone)]
struct ConfirmedCandidateRow {
    block_hash: String,
    block_number: u64,
    reward: Option<u128>,
}

fn to_u64_named(name: &str, value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|_| anyhow::anyhow!("{} overflow: {}", name, value))
}

fn to_i64_named(name: &str, value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| anyhow::anyhow!("{} overflow: {}", name, value))
}

fn parse_u128_named(name: &str, raw: &str) -> Result<u128> {
    raw.parse::<u128>()
        .map_err(|_| anyhow::anyhow!("invalid {} numeric value: {}", name, raw))
}

fn resolve_verify_range(
    db: &mut Client,
    verify_opt: &VerifySettlementOpt,
) -> Result<Option<(u64, u64)>> {
    let row = db.query_one(
        "select min(block_number), max(block_number)
         from pplns_candidates
         where status = 'confirmed'",
        &[],
    )?;
    let min_height_db = row.get::<_, Option<i64>>(0);
    let max_height_db = row.get::<_, Option<i64>>(1);
    let (min_height_db, max_height_db) = match (min_height_db, max_height_db) {
        (Some(min_v), Some(max_v)) => (
            to_u64_named("min confirmed height", min_v)?,
            to_u64_named("max confirmed height", max_v)?,
        ),
        _ => return Ok(None),
    };
    let from_height = verify_opt.from_height.unwrap_or(min_height_db);
    let to_height = verify_opt.to_height.unwrap_or(max_height_db);
    if from_height > to_height {
        return Err(anyhow::anyhow!(
            "invalid height range: from_height {} > to_height {}",
            from_height,
            to_height
        ));
    }
    Ok(Some((from_height, to_height)))
}

fn load_confirmed_candidates(
    db: &mut Client,
    from_height: u64,
    to_height: u64,
) -> Result<Vec<ConfirmedCandidateRow>> {
    let from_height = to_i64_named("from_height", from_height)?;
    let to_height = to_i64_named("to_height", to_height)?;
    let rows = db.query(
        "select block_hash, block_number, reward::text
         from pplns_candidates
         where status = 'confirmed'
           and block_number >= $1
           and block_number <= $2
         order by block_number asc, block_hash asc",
        &[&from_height, &to_height],
    )?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let reward_text = row.get::<_, Option<String>>(2);
        let reward = match reward_text {
            Some(raw) => Some(parse_u128_named("candidate.reward", &raw)?),
            None => None,
        };
        out.push(ConfirmedCandidateRow {
            block_hash: row.get(0),
            block_number: to_u64_named("candidate.block_number", row.get::<_, i64>(1))?,
            reward,
        });
    }
    Ok(out)
}

fn load_credit_sums_by_block(
    db: &mut Client,
    from_height: u64,
    to_height: u64,
) -> Result<HashMap<String, u128>> {
    let from_height = to_i64_named("from_height", from_height)?;
    let to_height = to_i64_named("to_height", to_height)?;
    let rows = db.query(
        "select c.block_hash, coalesce(sum(l.amount), 0)::text as total_credit
         from pplns_candidates c
         left join pplns_ledger_entries l
           on l.block_hash = c.block_hash and l.entry_type = 'credit'
         where c.block_number >= $1 and c.block_number <= $2
         group by c.block_hash",
        &[&from_height, &to_height],
    )?;
    let mut out = HashMap::with_capacity(rows.len());
    for row in rows {
        let block_hash = row.get::<_, String>(0);
        let sum_text = row.get::<_, String>(1);
        let amount = parse_u128_named("ledger sum(amount)", &sum_text)?;
        out.insert(block_hash, amount);
    }
    Ok(out)
}

fn check_confirmed_uniqueness(
    db: &mut Client,
    from_height: u64,
    to_height: u64,
) -> Result<Vec<String>> {
    let from_height = to_i64_named("from_height", from_height)?;
    let to_height = to_i64_named("to_height", to_height)?;
    let rows = db.query(
        "select block_number, count(*)::bigint
         from pplns_candidates
         where status = 'confirmed'
           and block_number >= $1
           and block_number <= $2
         group by block_number
         having count(*) > 1",
        &[&from_height, &to_height],
    )?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let block_number = to_u64_named("duplicate block_number", row.get::<_, i64>(0))?;
        let count = to_u64_named("duplicate confirmed count", row.get::<_, i64>(1))?;
        out.push(format!(
            "height {} has {} confirmed candidates (expected 1)",
            block_number, count
        ));
    }
    Ok(out)
}

fn check_non_confirmed_has_no_credit(
    db: &mut Client,
    from_height: u64,
    to_height: u64,
) -> Result<Vec<String>> {
    let from_height = to_i64_named("from_height", from_height)?;
    let to_height = to_i64_named("to_height", to_height)?;
    let rows = db.query(
        "select c.block_hash, c.status, sum(l.amount)::text
         from pplns_candidates c
         join pplns_ledger_entries l
           on l.block_hash = c.block_hash and l.entry_type = 'credit'
         where c.block_number >= $1
           and c.block_number <= $2
           and c.status <> 'confirmed'
         group by c.block_hash, c.status
         having sum(l.amount) <> 0",
        &[&from_height, &to_height],
    )?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let block_hash = row.get::<_, String>(0);
        let status = row.get::<_, String>(1);
        let amount = row.get::<_, String>(2);
        out.push(format!(
            "non-confirmed candidate has non-zero credit: block_hash={}, status={}, credit={}",
            block_hash, status, amount
        ));
    }
    Ok(out)
}

fn fetch_block_reward_sync(
    rpc: &dyn NodeRpcSync,
    block_hash: HashValue,
    block_number: u64,
) -> Result<u128> {
    let txn_infos = rpc.chain_get_block_txn_infos(block_hash)?;
    let mut reward = 0u128;
    for txn_info in txn_infos {
        let txn_hash = txn_info.transaction_hash;
        let events = rpc.chain_get_events_by_txn_hash(txn_hash, None)?;
        for event_info in events {
            let event = event_info.event;
            if event.block_hash != Some(block_hash) {
                continue;
            }
            let tag = event.type_tag.to_string();
            let data = &event.data.0;
            if let Some((reward_block_number, amount)) =
                PplnsRuntime::parse_block_reward_event(&tag, data)
            {
                if reward_block_number == block_number {
                    reward = reward.saturating_add(amount);
                }
            }
        }
    }
    Ok(reward)
}

pub fn run_verify_settlement(verify_opt: &VerifySettlementOpt) -> Result<()> {
    let mut db = Client::connect(verify_opt.pplns_database_url.as_str(), NoTls)
        .map_err(|err| anyhow::anyhow!("connect pplns database failed: {}", err))?;
    db.batch_execute(include_str!("../sql/pplns.postgresql.sql"))?;

    let Some((from_height, to_height)) = resolve_verify_range(&mut db, verify_opt)? else {
        println!("verify-settlement: no confirmed candidates found");
        return Ok(());
    };

    let confirmed = load_confirmed_candidates(&mut db, from_height, to_height)?;
    if confirmed.is_empty() {
        println!(
            "verify-settlement: no confirmed candidates in range [{}..={}]",
            from_height, to_height
        );
        return Ok(());
    }

    let mut issues = Vec::new();
    issues.extend(check_confirmed_uniqueness(&mut db, from_height, to_height)?);
    issues.extend(check_non_confirmed_has_no_credit(
        &mut db,
        from_height,
        to_height,
    )?);

    let credit_sums = load_credit_sums_by_block(&mut db, from_height, to_height)?;

    let rpc = build_sync_rpc_client(verify_opt.node_rpc.as_str())?;

    for candidate in confirmed {
        let Some(db_reward) = candidate.reward else {
            issues.push(format!(
                "candidate reward is null: block_hash={}, height={}",
                candidate.block_hash, candidate.block_number
            ));
            continue;
        };

        let main_block = match rpc.chain_get_block_by_number(candidate.block_number, None)? {
            Some(block) => block,
            None => {
                issues.push(format!(
                    "chain missing block by number: height={}, candidate_hash={}",
                    candidate.block_number, candidate.block_hash
                ));
                continue;
            }
        };
        let main_hash = main_block.header.block_hash;
        let candidate_hash = match HashValue::from_str(candidate.block_hash.as_str()) {
            Ok(hash) => hash,
            Err(err) => {
                issues.push(format!(
                    "invalid candidate hash format: block_hash={}, err={}",
                    candidate.block_hash, err
                ));
                continue;
            }
        };
        if candidate_hash != main_hash {
            issues.push(format!(
                "candidate hash not on main chain: height={}, candidate_hash={}, main_hash={}",
                candidate.block_number, candidate_hash, main_hash
            ));
            continue;
        }

        let chain_reward = fetch_block_reward_sync(&rpc, main_hash, candidate.block_number)?;
        if db_reward != chain_reward {
            issues.push(format!(
                "reward mismatch: height={}, block_hash={}, db_reward={}, chain_reward={}",
                candidate.block_number, candidate.block_hash, db_reward, chain_reward
            ));
        }

        let ledger_sum = credit_sums.get(&candidate.block_hash).copied().unwrap_or(0);
        if ledger_sum != db_reward {
            issues.push(format!(
                "ledger mismatch: height={}, block_hash={}, db_reward={}, ledger_sum={}",
                candidate.block_number, candidate.block_hash, db_reward, ledger_sum
            ));
        }
    }

    if issues.is_empty() {
        println!(
            "verify-settlement OK: range=[{}..={}], checked candidates and ledger consistency",
            from_height, to_height
        );
        return Ok(());
    }

    println!(
        "verify-settlement FAILED: range=[{}..={}], issues={}",
        from_height,
        to_height,
        issues.len()
    );
    for (idx, issue) in issues.iter().take(50).enumerate() {
        println!("{}. {}", idx + 1, issue);
    }
    Err(anyhow::anyhow!(
        "verify-settlement detected {} issue(s)",
        issues.len()
    ))
}
