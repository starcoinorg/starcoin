// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// vm2 txn factory
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
use anyhow::{anyhow, Context, Result};
use csv::{ReaderBuilder, WriterBuilder};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::{error, info, warn};
use starcoin_rpc_client::{AsyncRemoteStateReader, AsyncRpcClient, StateRootOption};
use starcoin_vm2_transaction_builder::{build_transfer_txn, DEFAULT_EXPIRATION_TIME};
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::view::{TransactionInfoView, TransactionStatusView};
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::path::Path;
use tokio::fs;
use tokio::fs::File;
use tokio::time::{sleep, Duration};

const DEFAULT_PASSWORD: &str = "password"; // Default password for new accounts
const INITIAL_BALANCE: u128 = 1_000_000_000;
const DEFAULT_AMOUNT: u128 = 1_000; // Default amount to transfer
const MIN_GAS_AMOUNT: u64 = 10_000_000; // max gas

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AccountEntry {
    address: AccountAddress,
    password: String,
}

#[derive(Debug, Clone)]
struct TxnRecord {
    txn_hash: HashValue,
    account_addr: AccountAddress,
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
    let address = client.account_create(password.to_owned()).await?.address;
    let entry = AccountEntry {
        address,
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
                account.address,
                account.password.clone(),
                Duration::from_secs(30),
            )
            .await
            .is_ok();
        if ok {
            info!("Unlocked account {}", account.address);
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
    let state_reader = AsyncRemoteStateReader::create(client, StateRootOption::Latest).await?;
    let bal = state_reader
        .get_balance(account.address)
        .await?
        .unwrap_or(0);
    if bal >= min_balance {
        return Ok(());
    }
    let need = min_balance - bal;
    info!(
        "Topping up {} with {} nano STC from {}",
        account.address, need, funding.address
    );
    let node_info = client.node_info().await?;
    let chain_id = node_info.net.chain_id().id();
    let timestamp = node_info.now_seconds + DEFAULT_EXPIRATION_TIME;
    // unlock funding account
    unlock_account(client, funding).await?;
    // build & send funding txn (sync call for simplicity)
    let txn_hash =
        create_and_submit(client, funding, account.address, need, timestamp, chain_id).await?;
    wait_txn_confirmed(client, txn_hash).await
}

/// Wait for txn to be confirmed.
async fn wait_txn_confirmed(client: &AsyncRpcClient, hash: HashValue) -> Result<()> {
    // simple polling; in production, prefer subscription.
    for _ in 0..30 {
        if let Some(info) = client.chain_get_transaction_info(hash).await? {
            return {
                match info.status {
                    TransactionStatusView::Executed => (),
                    _ => warn!("txn {:?} not executed yet", hash),
                };
                Ok(())
            };
        }
        sleep(Duration::from_secs(2)).await;
    }
    Err(anyhow!("txn {:?} not confirmed in time", hash))
}

/// Build, sign & submit a raw transfer txn.
async fn create_and_submit(
    client: &AsyncRpcClient,
    from: &AccountEntry,
    to: AccountAddress,
    amount: u128,
    timestamp: u64,
    chain_id: u8,
) -> Result<HashValue> {
    let seq_num = match client.next_sequence_number_in_txpool(from.address).await? {
        Some(num) => num,
        None => {
            let state_reader =
                AsyncRemoteStateReader::create(client, StateRootOption::Latest).await?;
            let acc = state_reader.get_account_resource(&from.address).await?;
            acc.map(|r| r.sequence_number()).unwrap_or(0)
        }
    };
    let raw = build_transfer_txn(
        from.address,
        to,
        seq_num,
        amount,
        1,              // gas price
        MIN_GAS_AMOUNT, // max gas amount
        timestamp,
        chain_id.into(), // chain ID
    );
    let signed = client.account_sign_txn(raw, from.address).await?;
    let hash = client.submit_txn(signed).await?;
    Ok(hash)
}

async fn generate_accounts(
    client: &AsyncRpcClient,
    csv_path: &str,
    count: usize,
    password: Option<&str>,
) -> Result<()> {
    let existed_accounts = load_accounts(csv_path)?;
    let mut changed = false;
    let mut filtered_accounts: Vec<AccountEntry> = Vec::new();
    for account in existed_accounts {
        let unlocked = unlock_account(client, &account).await.is_ok();
        if unlocked {
            filtered_accounts.push(account);
        } else {
            warn!(
                "Failed to unlock account {}, removing from list",
                account.address
            );
            changed = true;
        }
    }
    let existed = filtered_accounts.len();
    if changed {
        let file = File::open(csv_path).await?;
        file.set_len(0).await?;
        drop(file);
        for account in filtered_accounts {
            append_account(csv_path, &account)?;
        }
    }
    // todo: handle duplicated accounts
    let password = password.unwrap_or(DEFAULT_PASSWORD);
    for _ in 0..count - existed {
        let entry = create_account(client, csv_path, password).await?;
        info!("Created account {}", entry.address);
    }
    Ok(())
}

async fn generate_cmd(mut args: impl Iterator<Item = String>) -> Result<()> {
    let node_url = args.next().context("node url")?;
    let csv_path = args.next().context("csv path")?;
    let count: usize = args.next().context("count")?.parse()?;
    let password = args.next();
    let client = AsyncRpcClient::new(node_url.into()).await?;
    if fs::try_exists(&csv_path).await? && !fs::metadata(&csv_path).await?.is_file() {
        return Err(anyhow!("{} is not a file", csv_path));
    }
    generate_accounts(&client, &csv_path, count, password.as_deref()).await
}

#[tokio::main]
async fn main() -> Result<()> {
    starcoin_logger::init();
    let mut args = std::env::args().skip(1);
    let sub_cmd = args.next().context("sub command")?;
    match sub_cmd.as_str() {
        "generate" => {
            return generate_cmd(args).await;
        }
        "run" => (),
        _ => return Err(anyhow!("Unknown command: {}", sub_cmd)),
    }
    let node_url = args.next().context("node url")?;
    let csv_path = args.next().context("csv path")?;
    let funding_addr = args.next().context("funding addr")?;
    let funding_pw = args.next().context("funding pw")?;
    let target_addr = args.next().context("target addr")?;
    let min_balance: u128 = args
        .next()
        .map(|x| x.parse::<u128>())
        .transpose()?
        .unwrap_or(INITIAL_BALANCE);

    let client = AsyncRpcClient::new(node_url.into()).await?;
    let mut accounts = load_accounts(&csv_path)?;

    let funding_addr =
        AccountAddress::from_hex_literal(&funding_addr).context("Invalid funding address")?;
    let target_addr =
        AccountAddress::from_hex_literal(&target_addr).context("Invalid target address")?;

    // make sure funding account exists in list (for easy unlock later)
    if !accounts.iter().any(|a| a.address == funding_addr) {
        accounts.push(AccountEntry {
            address: funding_addr,
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
            error!("unlock failed {e}");
            continue;
        }

        // ensure balance
        if let Err(e) = ensure_balance(&client, &picked, &funding, min_balance).await {
            error!("balance top‑up failed {e}");
            continue;
        }

        let node_info = client.node_info().await?;
        let chain_id = node_info.net.chain_id().id();
        let timestamp = node_info.now_seconds + DEFAULT_EXPIRATION_TIME;
        // create & submit txn to target
        match create_and_submit(
            &client,
            &picked,
            target_addr,
            DEFAULT_AMOUNT,
            timestamp,
            chain_id,
        )
        .await
        {
            Ok(hash) => {
                info!("submitted txn {hash}");
                queue.push_back(TxnRecord {
                    txn_hash: hash,
                    account_addr: picked.address,
                    finished: false,
                });
            }
            Err(e) => {
                error!("submit failed {e}");
            }
        }

        // very naive subscription replacement: mark finished txns
        for rec in queue.iter_mut() {
            if !rec.finished {
                if let Some(TransactionInfoView { status, .. }) =
                    client.chain_get_transaction_info(rec.txn_hash).await?
                {
                    match status {
                        TransactionStatusView::Executed => {
                            info!("txn executed {:?}", rec.txn_hash);
                        }
                        TransactionStatusView::OutOfGas => {
                            warn!("txn expired or discarded {:?}", rec.txn_hash);
                        }
                        _ => (),
                    }
                }
                rec.finished = true;
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}
