// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// account_txn_manager.rs
// A minimal prototype that demonstrates the workflow described:
// 1. Load accounts from CSV (AccountAddress,Password)
// 2. Create & persist new accounts
// 3. Randomly pick an account, ensure no unfinished txns, then:
//    a. unlock
//    b. check / top‑up balance
//    c. build, sign & submit a transfer txn
// 4. Maintain an in‑memory queue (txn_hash, account, is_finished)
//    and mark items as finished on the event stream.
//
// NOTE: Built for Starcoin (https://github.com/starcoinorg/starcoin) JSON‑RPC.
// Adjust RPC endpoints or helper calls if you target a different chain.
//
// ───── Dependencies (add to Cargo.toml) ─────────────────────────────
// [dependencies]
// anyhow = "1"
// csv = "1"
// rand = "0.8"
// serde = { version = "1", features = ["derive"] }
// starcoin-crypto = "1"
// starcoin-rpc-client = "1"
// tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
// tracing = "0.1"
//
// Compile with: cargo run --release -- <CSV_PATH> <NODE_URL> <FUNDING_ACCOUNT_ADDR> <FUNDING_ACCOUNT_PASS> <TARGET_ADDR> <MIN_BALANCE_STC>
// Example: cargo run -- accounts.csv http://localhost:9850 0x... pwd 0xTARGET 1000000000

use anyhow::{anyhow, Context, Result};
use csv::{ReaderBuilder, WriterBuilder};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_crypto::HashValue;
use starcoin_crypto::SigningKey;
use starcoin_logger::prelude::{error, info, warn};
use starcoin_rpc_client::AsyncRpcClient;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::view::TransactionInfoView;
use std::collections::{HashMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tokio::time::{sleep, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AccountEntry {
    address: AccountAddress,
    password: String,
}

#[derive(Debug, Clone)]
struct TxnRecord {
    txn_hash: HashValue,
    account_addr: String,
    finished: bool,
}

/// Load AccountAddress,Password tuples from csv file.
fn load_accounts<P: AsRef<Path>>(path: P) -> Result<Vec<AccountEntry>> {
    if !path.as_ref().exists() {
        return Ok(Vec::new());
    }
    let mut rdr = ReaderBuilder::new().has_headers(false).from_path(path)?;
    let mut out = Vec::new();
    for result in rdr.deserialize() {
        let entry: AccountEntry = result?;
        out.push(entry);
    }
    Ok(out)
}

/// Append a new account line to csv.
fn append_account<P: AsRef<Path>>(path: P, account: &AccountEntry) -> Result<()> {
    let mut wtr = WriterBuilder::new()
        .has_headers(false)
        .from_writer(OpenOptions::new().create(true).append(true).open(path)?);
    wtr.serialize(account)?;
    wtr.flush()?;
    Ok(())
}

/// Create a fresh account locally and persist its credentials.
async fn create_account(
    client: &AsyncRpcClient,
    csv_path: &str,
    password: &str,
) -> Result<AccountEntry> {
    // Generate a new mnemonic / keypair via RPC (or locally, then import).
    let addr = client
        .account_create(Some(password.to_string()), None)
        .await?
        .account
        .ok_or_else(|| anyhow!("No address returned"))?;
    let entry = AccountEntry {
        address: format!("0x{}", addr.to_hex()),
        password: password.to_string(),
    };
    append_account(csv_path, &entry)?;
    Ok(entry)
}

/// Unlock an account; retry a few times on failure.
async fn unlock_account(client: &AsyncRpcClient, account: &AccountEntry) -> Result<()> {
    const MAX_RETRY: usize = 3;
    for _ in 0..MAX_RETRY {
        let ok = client
            .account_unlock(
                account.address.clone(),
                Some(account.password.clone()),
                None,
            )
            .await?;
        if ok {
            return Ok(());
        }
        warn!("unlock failed, retrying...");
        sleep(Duration::from_millis(500)).await;
    }
    Err(anyhow!("unable to unlock account {}", account.address))
}

/// Ensure balance >= min_balance (in STC nano).
async fn ensure_balance(
    client: &AsyncRpcClient,
    account: &AccountEntry,
    funding: &AccountEntry,
    min_balance: u128,
) -> Result<()> {
    let bal = client
        .state_get_balance(account.address.clone(), None)
        .await?;
    if bal.map_or(0, |b| b.value) >= min_balance {
        return Ok(());
    }
    let need = min_balance - bal.map_or(0, |b| b.value);
    info!("Topping up {} with {} nano STC", account.address, need);
    // build & send funding txn (sync call for simplicity)
    let txn_hash = client
        .account_transfer(
            funding.address.clone(),
            funding.password.clone(),
            account.address.clone(),
            need,
            None,
            None,
        )
        .await?;
    wait_txn_confirmed(client, txn_hash).await
}

/// Wait for txn to be confirmed.
async fn wait_txn_confirmed(client: &AsyncRpcClient, hash: HashValue) -> Result<()> {
    // simple polling; in production, prefer subscription.
    for _ in 0..30 {
        if let Some(info) = client.chain_get_transaction_info(hash).await? {
            if info.status == "Executed" {
                return Ok(());
            }
        }
        sleep(Duration::from_secs(2)).await;
    }
    Err(anyhow!("txn {:?} not confirmed in time", hash))
}

/// Build, sign & submit a raw transfer txn.
async fn create_and_submit(
    client: &AsyncRpcClient,
    from: &AccountEntry,
    to: &str,
    amount: u128,
) -> Result<HashValue> {
    let raw = client
        .txpool_generate_transfer_raw_txn(from.address.clone(), to.to_string(), amount, None, None)
        .await?;
    let signed = client
        .account_sign_txn(from.address.clone(), from.password.clone(), raw, None)
        .await?;
    let hash = client.submit_txn(signed).await?;
    Ok(hash)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let csv_path = args.next().context("csv path")?;
    let node_url = args.next().context("node url")?;
    let funding_addr = args.next().context("funding addr")?;
    let funding_pw = args.next().context("funding pw")?;
    let target_addr = args.next().context("target addr")?;
    let min_balance: u128 = args.next().context("min balance")?.parse()?;

    let client = AsyncRpcClient::new(node_url.into())?;
    let mut accounts = load_accounts(&csv_path)?;

    // make sure funding account exists in list (for easy unlock later)
    if !accounts.iter().any(|a| a.address == funding_addr) {
        accounts.push(AccountEntry {
            address: funding_addr.clone(),
            password: funding_pw.clone(),
        });
    }
    let funding = accounts
        .iter()
        .find(|a| a.address == funding_addr)
        .unwrap()
        .clone();

    // Txn queue
    let mut queue: VecDeque<TxnRecord> = VecDeque::new();

    loop {
        // housekeeping: drop finished txns
        queue.retain(|rec| !rec.finished);

        // pick a random account (skip funding)
        let candidates: Vec<_> = accounts
            .iter()
            .filter(|a| a.address != funding_addr)
            .collect();
        if candidates.is_empty() {
            error!("no candidate accounts, sleeping...");
            sleep(Duration::from_secs(10)).await;
            continue;
        }
        let picked = (*candidates.choose(&mut rand::thread_rng()).unwrap()).clone();

        // Ensure no unfinished txn for this account
        if queue.iter().any(|r| r.account_addr == picked.address) {
            info!("{} still has pending txn, skipping", picked.address);
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        // try unlock
        if let Err(e) = unlock_account(&client, &picked).await {
            error!(?e, "unlock failed");
            continue;
        }

        // ensure balance
        if let Err(e) = ensure_balance(&client, &picked, &funding, min_balance).await {
            error!(?e, "balance top‑up failed");
            continue;
        }

        // create & submit txn to target
        match create_and_submit(&client, &picked, &target_addr, 1_000_000).await {
            Ok(hash) => {
                info!(?hash, "submitted txn");
                queue.push_back(TxnRecord {
                    txn_hash: hash,
                    account_addr: picked.address.clone(),
                    finished: false,
                });
            }
            Err(e) => {
                error!(?e, "submit failed");
            }
        }

        // very naive subscription replacement: mark finished txns
        for rec in queue.iter_mut() {
            if !rec.finished {
                if let Some(TransactionInfoView { status, .. }) =
                    client.chain_get_transaction_info(rec.txn_hash).await?
                {
                    if status == "Executed" {
                        rec.finished = true;
                    }
                }
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}
