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
use starcoin_logger::prelude::{info, warn};
use starcoin_rpc_client::{AsyncRemoteStateReader, AsyncRpcClient, StateRootOption};
use starcoin_vm2_transaction_builder::{build_transfer_txn, DEFAULT_EXPIRATION_TIME};
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::view::TransactionStatusView;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::sync::mpsc;
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

enum AccountState {
    Initial,
    Ready,
    Submitted(HashValue),
    Finished,
    Error(String),
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
    account: AccountAddress,
    funding: &AccountEntry,
    min_balance: u128,
) -> Result<()> {
    let state_reader = AsyncRemoteStateReader::create(client, StateRootOption::Latest).await?;
    let bal = state_reader.get_balance(account).await?.unwrap_or(0);
    if bal >= min_balance {
        return Ok(());
    }
    let need = min_balance * 10 - bal;
    info!(
        "Topping up {} with {} nano STC from {}",
        account, need, funding.address
    );
    let node_info = client.node_info().await?;
    let chain_id = node_info.net.chain_id().id();
    let timestamp = node_info.now_seconds + DEFAULT_EXPIRATION_TIME;
    // unlock funding account
    unlock_account(client, funding).await?;
    // build & send funding txn (sync call for simplicity)
    let txn_hash = create_and_submit(client, funding, account, need, timestamp, chain_id).await?;
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
        let file = File::create(csv_path).await?;
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

async fn account_worker(
    client: Arc<AsyncRpcClient>,
    entry: AccountEntry,
    target_addr: AccountAddress,
    min_balance: u128,
    tx_amount: u128,
    tx: mpsc::Sender<AccountAddress>,
) {
    let mut state = AccountState::Initial;
    loop {
        match &state {
            AccountState::Initial => {
                let state_reader = AsyncRemoteStateReader::create(&client, StateRootOption::Latest)
                    .await
                    .unwrap();
                let bal = state_reader
                    .get_balance(entry.address)
                    .await
                    .unwrap()
                    .unwrap_or(0);
                if bal >= min_balance {
                    state = AccountState::Ready;
                    continue;
                }
                tx.send(entry.address).await.unwrap();
                sleep(Duration::from_secs(2)).await;
            }
            AccountState::Ready => {
                let node_info = match client.node_info().await {
                    Ok(info) => info,
                    Err(e) => {
                        warn!("failed to get node info {e}");
                        state = AccountState::Error(format!("node info: {e}"));
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };
                if let Err(e) = unlock_account(&client, &entry).await {
                    warn!("unlock error {e}");
                    state = AccountState::Error(format!("unlock: {e}"));
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                let chain_id = node_info.net.chain_id().id();
                let timestamp = node_info.now_seconds + DEFAULT_EXPIRATION_TIME;
                match create_and_submit(
                    &client,
                    &entry,
                    target_addr,
                    tx_amount,
                    timestamp,
                    chain_id,
                )
                .await
                {
                    Ok(hash) => {
                        info!("submitted txn {hash} for {}", entry.address);
                        state = AccountState::Submitted(hash);
                    }
                    Err(e) => {
                        warn!("submit error {e}");
                        state = AccountState::Error(format!("submit: {e}"));
                    }
                }
            }
            AccountState::Submitted(hash) => match client.chain_get_transaction_info(*hash).await {
                Ok(Some(info)) => {
                    match info.status {
                        TransactionStatusView::Executed => {
                            info!("txn executed {:?}", info.transaction_hash);
                        }
                        TransactionStatusView::OutOfGas => {
                            warn!(
                                "txn executed failed because of OutOfGas {:?}",
                                info.transaction_hash
                            );
                        }
                        _ => (),
                    }
                    state = AccountState::Finished;
                }
                Ok(None) => {
                    sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    warn!("poll error {e}");
                    state = AccountState::Error(format!("poll: {e}"));
                }
            },
            AccountState::Finished => {
                info!("test cycle finished → restarting {}", entry.address);
                state = AccountState::Initial; // repeat endlessly; remove to finish once
            }
            AccountState::Error(e) => {
                warn!("error state: {e}, retrying in 1s");
                sleep(Duration::from_secs(1)).await;
                state = AccountState::Initial; // reset to initial state
            }
        }
    }
}
async fn balancer_worker(
    client: Arc<AsyncRpcClient>,
    funding: AccountEntry,
    min_balance: u128,
    mut rx: mpsc::Receiver<AccountAddress>,
) {
    while let Some(account) = rx.recv().await {
        if let Err(e) = ensure_balance(&client, account, &funding, min_balance).await {
            warn!("balancer error {e}");
        }
    }
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

    let client = Arc::new(AsyncRpcClient::new(node_url.into()).await?);
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

    let mut test_accounts = accounts
        .iter()
        .filter(|a| a.address != funding_addr)
        .cloned()
        .collect::<Vec<_>>();
    test_accounts.shuffle(&mut rand::thread_rng());

    let (tx, rx) = mpsc::channel(10240);

    let mut handles = Vec::new();
    for entry in test_accounts {
        let handle = tokio::spawn({
            let client = Arc::clone(&client);
            let tx_amount = DEFAULT_AMOUNT;
            let tx = tx.clone();
            async move {
                account_worker(client, entry, target_addr, min_balance, tx_amount, tx).await;
            }
        });
        handles.push(handle);
    }
    let balancer = tokio::spawn({
        let client = Arc::clone(&client);
        let funding = funding.clone();
        async move {
            balancer_worker(client, funding, min_balance, rx).await;
        }
    });
    handles.push(balancer);

    futures::future::join_all(handles).await;
    Ok(())
}
