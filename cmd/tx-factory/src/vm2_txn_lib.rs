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
use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use starcoin_logger::prelude::{info, warn};
use starcoin_rpc_client::{AsyncRemoteStateReader, AsyncRpcClient, StateRootOption};
use starcoin_vm2_account_api::{AccountPrivateKey, AccountPublicKey};
use starcoin_vm2_crypto::{keygen::KeyGen, HashValue, ValidCryptoMaterialStringExt};
use starcoin_vm2_transaction_builder::{build_transfer_txn, DEFAULT_EXPIRATION_TIME};
use starcoin_vm2_types::account_address;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm2_types::view::TransactionStatusView;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

const INITIAL_BALANCE: u128 = 1_000_000_000;
const DEFAULT_AMOUNT: u128 = 1_000; // Default amount to transfer
const MIN_GAS_AMOUNT: u64 = 10_000_000; // max gas

//"ok": {
//   "account": "0x047e2d5eeb825c80ffa986b6cd0b521d",
//   "private_key": "0x57bc2570de3bfe939ad6127d17d5b81db99a4bf4282cea5406fb7149e7ae67c5"
// }
pub static FUNDING_ACCOUNT: Lazy<AccountEntry> = Lazy::new(|| {
    let private_key_str = "0x57bc2570de3bfe939ad6127d17d5b81db99a4bf4282cea5406fb7149e7ae67c5";
    let private_key = AccountPrivateKey::from_encoded_string(private_key_str)
        .expect("Invalid funding private key");
    let public_key = private_key.public_key();
    let address = public_key.derived_address();
    AccountEntry {
        address,
        public_key,
        private_key,
    }
});

// 在 async 函数中直接用 &*FUNDING_ACCOUNT 即可
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountEntry {
    address: AccountAddress,
    public_key: AccountPublicKey,
    private_key: AccountPrivateKey,
}

impl AccountEntry {
    pub fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        let signature = self.private_key.sign(&raw_txn)?;
        Ok(SignedUserTransaction::new(raw_txn, signature))
    }
    pub fn address(&self) -> AccountAddress {
        self.address
    }
}

enum AccountState {
    Initial,
    Ready,
    Submitted(HashValue),
    Finished,
    Error(String),
}

/// Load Account from file
async fn load_accounts<P: AsRef<Path>>(path: P) -> Result<Vec<AccountEntry>> {
    let file = match File::open(&path).await {
        Ok(f) => f,
        Err(_) => return Ok(Vec::new()),
    };
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut accounts = Vec::new();
    while let Some(line) = lines.next_line().await? {
        let private_key = AccountPrivateKey::from_encoded_string(&line)?;
        let ae = AccountEntry {
            address: private_key.public_key().derived_address(),
            public_key: private_key.public_key(),
            private_key,
        };
        accounts.push(ae);
    }

    Ok(accounts)
}

/// Append a new account line.
async fn append_account<P: AsRef<Path>>(path: P, account: &AccountEntry) -> Result<()> {
    let encoded = account.private_key.to_encoded_string()?;
    let mut file = File::options().append(true).create(true).open(path).await?;
    file.write_all(format!("{}\n", encoded).as_bytes()).await?;
    Ok(())
}

/// Create a fresh account locally
async fn create_account(account_path: &str) -> Result<AccountEntry> {
    let mut key_gen = KeyGen::from_os_rng();
    let (private_key, public_key) = key_gen.generate_keypair();
    let address = account_address::from_public_key(&public_key);
    let account_public_key = AccountPublicKey::Single(public_key);
    let entry = AccountEntry {
        address,
        public_key: account_public_key,
        private_key: AccountPrivateKey::Single(private_key),
    };
    append_account(account_path, &entry).await?;
    Ok(entry)
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
    let signed = from.sign_txn(raw)?;
    let hash = client.submit_txn(signed).await?;
    Ok(hash)
}

async fn account_get_balance(client: &AsyncRpcClient, address: AccountAddress) -> Result<u128> {
    let state_reader = AsyncRemoteStateReader::create(client, StateRootOption::Latest).await?;
    Ok(state_reader.get_balance(address).await?.unwrap_or(0))
}

async fn generate_accounts(account_path: &str, count: usize) -> Result<()> {
    let existed_accounts = load_accounts(account_path).await?;
    let existed = existed_accounts.len();
    // todo: handle duplicated accounts
    for _ in 0..count - existed {
        let entry = create_account(account_path).await?;
        info!("Created account {}", entry.address);
    }
    Ok(())
}

pub async fn generate_cmd(account_path: String, count: usize) -> Result<()> {
    if fs::try_exists(&account_path).await? && !fs::metadata(&account_path).await?.is_file() {
        return Err(anyhow!("{} is not a file", account_path));
    }
    generate_accounts(&account_path, count).await
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
                let bal = account_get_balance(&client, entry.address).await;
                let Ok(bal) = bal else {
                    warn!("Failed to get balance for {}", entry.address);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                };
                if bal >= min_balance {
                    state = AccountState::Ready;
                    continue;
                }
                if let Err(e) = tx.send(entry.address).await {
                    warn!("Failed to send account to get tokens: {e}");
                };
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
    funding: &AccountEntry,
    min_balance: u128,
    mut rx: mpsc::Receiver<AccountAddress>,
) {
    while let Some(account) = rx.recv().await {
        if let Err(e) = ensure_balance(&client, account, funding, min_balance).await {
            warn!("balancer error {e}");
        }
    }
}
pub async fn async_main(
    client: Arc<AsyncRpcClient>,
    target: AccountAddress,
    account_path: String,
) -> Result<()> {
    let min_balance: u128 = INITIAL_BALANCE;
    let funding = &*FUNDING_ACCOUNT;

    let mut accounts = load_accounts(&account_path).await?;
    accounts.shuffle(&mut rand::thread_rng());

    let (tx, rx) = mpsc::channel(10240);

    let mut handles = Vec::new();
    for entry in accounts {
        let handle = tokio::spawn({
            let client = Arc::clone(&client);
            let tx_amount = DEFAULT_AMOUNT;
            let tx = tx.clone();
            async move {
                account_worker(client, entry, target, min_balance, tx_amount, tx).await;
            }
        });
        handles.push(handle);
    }
    let balancer = tokio::spawn({
        let client = Arc::clone(&client);
        async move {
            balancer_worker(client, funding, min_balance, rx).await;
        }
    });
    handles.push(balancer);

    futures::future::join_all(handles).await;
    Ok(())
}
