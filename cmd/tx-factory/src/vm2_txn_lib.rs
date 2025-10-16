// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// vm2 txn factory
// A minimal prototype that demonstrates the workflow described:
// 1. Load accounts from SQLite store
// 2. Create & persist new accounts
// 3. Randomly pick an account, ensure no unfinished txns, then:
//    a. unlock
//    b. check / top‑up balance
//    c. build, sign & submit a transfer txn
// 4. Maintain an in‑memory queue (txn_hash, account, is_finished)
//    and mark items as finished on the event stream.
//
// Standard library
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
};

// Third-party crates
use anyhow::{anyhow, Result};
use futures::TryStreamExt;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use rusqlite::{params, Connection};
use tokio::{
    fs,
    sync::{mpsc, oneshot, OwnedSemaphorePermit, RwLock, Semaphore},
    task,
    time::{sleep, Duration, Instant},
};

// Starcoin crates
use starcoin_logger::prelude::{debug, info, warn};
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_client::{AsyncRemoteStateReader, AsyncRpcClient, StateRootOption};
use starcoin_transaction_builder::vm2::{build_transfer_txn, DEFAULT_EXPIRATION_TIME};
use starcoin_vm2_account_api::AccountPrivateKey;
use starcoin_vm2_crypto::{keygen::KeyGen, HashValue, ValidCryptoMaterialStringExt};
use starcoin_vm2_types::{
    account_address::{self, AccountAddress},
    transaction::{RawUserTransaction, SignedUserTransaction},
    view::TransactionStatusView,
};

const INITIAL_BALANCE: u128 = 1_000_000_000;
const DEFAULT_AMOUNT: u128 = 1_000; // Default amount to transfer
const MIN_GAS_AMOUNT: u64 = 10_000_000; // max gas

const METRICS_WINDOW: Duration = Duration::from_secs(60);
const METRICS_TICK: Duration = Duration::from_secs(5);
const TARGET_TXNS_PER_BLOCK: f64 = 200.0;
const TXN_PER_BLOCK_TOLERANCE: f64 = 0.1;
const MAX_CONFIRMATION_LATENCY: Duration = Duration::from_secs(8);
const CONCURRENCY_STEP: usize = 1;
const MIN_CONCURRENCY: usize = 1;
const MAX_CONCURRENCY: usize = 64;
const MIN_INTERVAL_STEP_MS: u64 = 100;
const MIN_INTERVAL_MIN_MS: u64 = 0;
const MIN_INTERVAL_MAX_MS: u64 = 2_000;
const INITIAL_CONCURRENCY: usize = 4;

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
        private_key,
    }
});

static GLOBAL_NODE_INFO: OnceLock<Arc<RwLock<NodeInfo>>> = OnceLock::new();

async fn node_info() -> (u8, u64) {
    let info = GLOBAL_NODE_INFO
        .get()
        .expect("GLOBAL_NODE_INFO uninitialized")
        .read()
        .await;
    (info.net.chain_id().id(), info.now_seconds)
}

fn set_info(info: NodeInfo) {
    GLOBAL_NODE_INFO
        .set(Arc::new(RwLock::new(info)))
        .expect("GLOBAL_NODE_INFO already initialized");
}

pub struct AccountEntry {
    address: AccountAddress,
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
    Submitted((HashValue, oneshot::Receiver<Result<()>>)),
    Finished,
    Error(String),
}

struct TxnReceipt {
    txn_hash: HashValue,
    response_tx: oneshot::Sender<Result<()>>,
}

#[derive(Clone, Debug)]
struct TxnLifecycleDurations {
    queue: Duration,
    build: Duration,
    submit: Duration,
    confirmation: Duration,
    total: Duration,
}

struct TxnLifecycle {
    account: AccountAddress,
    started_at: Instant,
    ready_at: Option<Instant>,
    submit_started_at: Option<Instant>,
    submitted_at: Option<Instant>,
    confirmed_at: Option<Instant>,
}

impl TxnLifecycle {
    fn new(account: AccountAddress) -> Self {
        let mut lifecycle = Self {
            account,
            started_at: Instant::now(),
            ready_at: None,
            submit_started_at: None,
            submitted_at: None,
            confirmed_at: None,
        };
        lifecycle.restart();
        lifecycle
    }

    fn restart(&mut self) {
        self.started_at = Instant::now();
        self.ready_at = None;
        self.submit_started_at = None;
        self.submitted_at = None;
        self.confirmed_at = None;
    }

    fn account(&self) -> AccountAddress {
        self.account
    }

    fn mark_ready(&mut self) {
        self.ready_at.get_or_insert_with(Instant::now);
    }

    fn mark_submit_started(&mut self) {
        self.submit_started_at.get_or_insert_with(Instant::now);
    }

    fn mark_submitted(&mut self) {
        self.submitted_at.get_or_insert_with(Instant::now);
    }

    fn mark_confirmed(&mut self) {
        self.confirmed_at.get_or_insert_with(Instant::now);
    }

    fn durations(&self) -> Option<TxnLifecycleDurations> {
        let ready = self.ready_at?;
        let submit_started = self.submit_started_at?;
        let submitted = self.submitted_at?;
        let confirmed = self.confirmed_at?;

        Some(TxnLifecycleDurations {
            queue: ready.saturating_duration_since(self.started_at),
            build: submit_started.saturating_duration_since(ready),
            submit: submitted.saturating_duration_since(submit_started),
            confirmation: confirmed.saturating_duration_since(submitted),
            total: confirmed.saturating_duration_since(self.started_at),
        })
    }
}

enum MetricsEvent {
    TxnLifecycle {
        txn_hash: HashValue,
        account: AccountAddress,
        durations: TxnLifecycleDurations,
    },
    BalanceTopUp {
        account: AccountAddress,
        amount: u128,
        duration: Duration,
    },
    BlockStats {
        block_hash: HashValue,
        block_number: u64,
        txn_count: usize,
        interval: Duration,
    },
}

struct AdaptiveLimiter {
    semaphore: Arc<Semaphore>,
    held: Mutex<Vec<OwnedSemaphorePermit>>,
    limit: AtomicUsize,
}

impl AdaptiveLimiter {
    fn new(initial_limit: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(initial_limit)),
            held: Mutex::new(Vec::new()),
            limit: AtomicUsize::new(initial_limit),
        }
    }

    async fn acquire(&self) -> OwnedSemaphorePermit {
        self.semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Semaphore closed")
    }

    async fn update_limit(&self, new_limit: usize) {
        let current = self.limit.load(Ordering::SeqCst);
        if new_limit == current {
            return;
        }
        if new_limit > current {
            let mut to_release = new_limit - current;
            let mut held = self.held.lock().expect("held mutex poisoned");
            while to_release > 0 {
                if let Some(permit) = held.pop() {
                    drop(permit);
                    to_release -= 1;
                } else {
                    self.semaphore.add_permits(to_release);
                    break;
                }
            }
        } else {
            let diff = current - new_limit;
            let mut newly_held = Vec::with_capacity(diff);
            for _ in 0..diff {
                let permit = self
                    .semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .expect("Semaphore closed while shrinking");
                newly_held.push(permit);
            }
            let mut held = self.held.lock().expect("held mutex poisoned");
            held.extend(newly_held);
        }
        self.limit.store(new_limit, Ordering::SeqCst);
    }

    fn limit(&self) -> usize {
        self.limit.load(Ordering::SeqCst)
    }
}
fn init_account_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS accounts (
            address TEXT PRIMARY KEY,
            private_key TEXT NOT NULL UNIQUE,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        "#,
    )?;
    Ok(())
}

async fn with_account_conn<T, F>(path: &Path, f: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&Connection) -> Result<T> + Send + 'static,
{
    let path = path.to_owned();
    task::spawn_blocking(move || -> Result<T> {
        let conn = Connection::open(&path)?;
        init_account_table(&conn)?;
        let result = f(&conn)?;
        Ok(result)
    })
    .await
    .map_err(|e| anyhow!("blocking SQLite task panicked: {}", e))?
}

async fn dedup_accounts<P: AsRef<Path>>(path: P) -> Result<usize> {
    let removed = with_account_conn(path.as_ref(), |conn| {
        let removed_by_address = conn.execute(
            r#"
            DELETE FROM accounts
            WHERE rowid NOT IN (
                SELECT MIN(rowid) FROM accounts GROUP BY address
            )
            "#,
            [],
        )?;
        let removed_by_key = conn.execute(
            r#"
            DELETE FROM accounts
            WHERE rowid NOT IN (
                SELECT MIN(rowid) FROM accounts GROUP BY private_key
            )
            "#,
            [],
        )?;
        Ok(removed_by_address + removed_by_key)
    })
    .await?;
    if removed > 0 {
        info!("Removed {} duplicated account entries", removed);
    }
    Ok(removed)
}

async fn account_count<P: AsRef<Path>>(path: P) -> Result<usize> {
    with_account_conn(path.as_ref(), |conn| {
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
        Ok(count as usize)
    })
    .await
}

/// Load accounts from SQLite store.
async fn load_accounts<P: AsRef<Path>>(path: P) -> Result<Vec<AccountEntry>> {
    let path_buf = path.as_ref().to_owned();
    dedup_accounts(&path_buf).await?;
    let rows: Vec<String> = with_account_conn(&path_buf, |conn| {
        let mut stmt =
            conn.prepare("SELECT private_key FROM accounts ORDER BY created_at ASC, address ASC")?;
        let keys = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(keys)
    })
    .await?;
    let mut accounts = Vec::with_capacity(rows.len());
    for encoded in rows {
        let private_key = AccountPrivateKey::from_encoded_string(&encoded)?;
        let address = private_key.public_key().derived_address();
        accounts.push(AccountEntry {
            address,
            private_key,
        });
    }
    Ok(accounts)
}

/// Append a new account entry. Returns `true` if insertion happened, `false` otherwise.
async fn append_account<P: AsRef<Path>>(path: P, account: &AccountEntry) -> Result<bool> {
    let encoded = account.private_key.to_encoded_string()?;
    let address = account.address.to_string();
    let inserted = with_account_conn(path.as_ref(), move |conn| {
        let affected = conn.execute(
            "INSERT OR IGNORE INTO accounts(address, private_key) VALUES (?1, ?2)",
            params![address, encoded],
        )?;
        Ok(affected > 0)
    })
    .await?;
    Ok(inserted)
}

/// Create a fresh account locally
async fn create_account(account_path: &str) -> Result<AccountEntry> {
    loop {
        let mut key_gen = KeyGen::from_os_rng();
        let (private_key, public_key) = key_gen.generate_keypair();
        let address = account_address::from_public_key(&public_key);
        let entry = AccountEntry {
            address,
            private_key: AccountPrivateKey::Single(private_key),
        };
        if append_account(account_path, &entry).await? {
            return Ok(entry);
        } else {
            warn!("Generated duplicate account {}, retrying", entry.address);
        }
    }
}

/// Ensure balance >= min_balance (in STC nano).
async fn ensure_balance(
    client: &AsyncRpcClient,
    account: AccountAddress,
    funding: &AccountEntry,
    min_balance: u128,
    metrics_tx: &mpsc::Sender<MetricsEvent>,
) -> Result<()> {
    let state_reader = AsyncRemoteStateReader::create(client, StateRootOption::Latest).await?;
    let bal = state_reader.get_balance(account).await?.unwrap_or(0);
    if bal >= min_balance {
        return Ok(());
    }
    let need = min_balance * 10 - bal;
    let start = Instant::now();
    info!(
        "Topping up {} with {} nano STC from {}",
        account, need, funding.address
    );
    let (chain_id, now_seconds) = node_info().await;
    let timestamp = now_seconds + DEFAULT_EXPIRATION_TIME;
    // build & send funding txn (sync call for simplicity)
    let txn_hash = create_and_submit(client, funding, account, need, timestamp, chain_id).await?;
    wait_txn_confirmed(client, txn_hash).await?;
    let duration = start.elapsed();
    if let Err(e) = metrics_tx
        .send(MetricsEvent::BalanceTopUp {
            account,
            amount: need,
            duration,
        })
        .await
    {
        warn!("Failed to record funding metrics: {e}");
    }
    Ok(())
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
    dedup_accounts(account_path).await?;
    let existed = account_count(account_path).await?;
    if existed >= count {
        info!(
            "Account store already has {} entries, target {} -> nothing to create",
            existed, count
        );
        return Ok(());
    }
    let to_create = count - existed;
    for _ in 0..to_create {
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
    balancer_tx: mpsc::Sender<AccountAddress>,
    confirmer_tx: mpsc::Sender<TxnReceipt>,
    limiter: Arc<AdaptiveLimiter>,
    min_submit_interval_ms: Arc<AtomicU64>,
    metrics_tx: mpsc::Sender<MetricsEvent>,
) {
    let mut state = AccountState::Initial;
    let mut lifecycle = TxnLifecycle::new(entry.address);
    let mut limiter_permit: Option<OwnedSemaphorePermit> = None;
    let mut last_submission = Instant::now() - Duration::from_secs(60);
    loop {
        match &mut state {
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
                if let Err(e) = balancer_tx.send(entry.address).await {
                    warn!("Failed to send account to get tokens: {e}");
                };
                sleep(Duration::from_secs(2)).await;
            }
            AccountState::Ready => {
                lifecycle.mark_ready();
                if limiter_permit.is_none() {
                    limiter_permit = Some(limiter.acquire().await);
                }
                let interval_ms = min_submit_interval_ms.load(Ordering::Relaxed);
                if interval_ms > 0 {
                    let interval = Duration::from_millis(interval_ms);
                    let elapsed = last_submission.elapsed();
                    if elapsed < interval {
                        sleep(interval - elapsed).await;
                    }
                }
                lifecycle.mark_submit_started();
                let (chain_id, now_seconds) = node_info().await;
                let timestamp = now_seconds + DEFAULT_EXPIRATION_TIME;
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
                        lifecycle.mark_submitted();
                        info!("submitted txn {hash} for {}", entry.address);
                        let (tx_receipt, rx) = oneshot::channel();
                        if let Err(e) = confirmer_tx
                            .send(TxnReceipt {
                                txn_hash: hash,
                                response_tx: tx_receipt,
                            })
                            .await
                        {
                            warn!("Failed to send txn receipt: {e}");
                            state = AccountState::Error("receipt channel closed".to_string());
                            continue;
                        }
                        state = AccountState::Submitted((hash, rx));
                    }
                    Err(e) => {
                        warn!("submit error {e}");
                        if let Some(permit) = limiter_permit.take() {
                            drop(permit);
                        }
                        lifecycle.restart();
                        state = AccountState::Error(format!("submit: {e}"));
                    }
                }
            }
            AccountState::Submitted((txn_hash, rx)) => match rx.try_recv() {
                Ok(_) => {
                    info!("txn {txn_hash} confirmed for {}", entry.address);
                    lifecycle.mark_confirmed();
                    last_submission = Instant::now();
                    if let Some(durations) = lifecycle.durations() {
                        if let Err(e) = metrics_tx
                            .send(MetricsEvent::TxnLifecycle {
                                txn_hash: *txn_hash,
                                account: lifecycle.account(),
                                durations,
                            })
                            .await
                        {
                            warn!("Failed to record txn metrics: {e}");
                        }
                    }
                    state = AccountState::Finished;
                }
                Err(oneshot::error::TryRecvError::Empty) => {
                    sleep(Duration::from_millis(100)).await;
                }
                Err(_) => {
                    warn!("failed to receive confirmation for txn {txn_hash}");
                    state = AccountState::Error("confirmation channel closed".to_string());
                }
            },
            AccountState::Finished => {
                info!("test cycle finished → restarting {}", entry.address);
                if let Some(permit) = limiter_permit.take() {
                    drop(permit);
                }
                lifecycle.restart();
                state = AccountState::Initial; // repeat endlessly; remove to finish once
            }
            AccountState::Error(e) => {
                warn!("error state: {e}, retrying in 1s");
                sleep(Duration::from_secs(1)).await;
                if let Some(permit) = limiter_permit.take() {
                    drop(permit);
                }
                lifecycle.restart();
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
    metrics_tx: mpsc::Sender<MetricsEvent>,
) {
    while let Some(account) = rx.recv().await {
        if let Err(e) = ensure_balance(&client, account, funding, min_balance, &metrics_tx).await {
            warn!("balancer error {e}");
        }
    }
}
async fn txn_confirmer(
    client: Arc<AsyncRpcClient>,
    mut rx: mpsc::Receiver<TxnReceipt>,
    metrics_tx: mpsc::Sender<MetricsEvent>,
) {
    let mut confirmed_txns = BTreeSet::new();
    let mut unconfirmed_txns = BTreeMap::new();
    let mut last_block_timestamp: Option<u64> = None;

    loop {
        let Ok(mut stream) = client.subscribe_new_blocks().await else {
            warn!("Failed to subscribe to new blocks");
            sleep(Duration::from_secs(30)).await;
            continue;
        };

        loop {
            tokio::select! {
                Some(receipt) = rx.recv() => {
                    let txn_hash = receipt.txn_hash;
                    if confirmed_txns.remove(&txn_hash) {
                       receipt.response_tx.send(Ok(())).expect("Failed to send confirmation for txn");
                    } else {
                        unconfirmed_txns.insert(txn_hash, receipt);
                    }
                }
                v = stream.try_next() => {
                    match v {
                        Ok(None) => break,
                        Ok(Some(event)) => {
                            let mut txns = event.body.txn_hashes();
                            let txn_count = txns.len();
                            let current_ts = event.header.timestamp.0;
                            let interval = last_block_timestamp
                                .map(|prev_ts| Duration::from_millis(current_ts.saturating_sub(prev_ts)))
                                .unwrap_or_else(|| Duration::from_millis(0));
                            if let Err(e) = metrics_tx
                                .send(MetricsEvent::BlockStats {
                                    block_hash: event.header.block_hash,
                                    block_number: event.header.number.0,
                                    txn_count,
                                    interval,
                                })
                                .await
                            {
                                warn!("Failed to emit block metrics: {e}");
                            }
                            last_block_timestamp = Some(current_ts);
                            txns.retain(|hash| {
                                match unconfirmed_txns.remove(hash) { Some(receipt) => {
                                    receipt.response_tx.send(Ok(())).expect("Failed to send confirmation for txn");
                                    false
                                } _ => {
                                    true
                                }}
                            });
                            confirmed_txns.extend(txns);
                        }
                        Err(e) => {
                            warn!("Error receiving new block event: {}", e);
                            break; // Exit the inner loop to re-subscribe
                        }
                    }
                }

            }
        }
    }
}

fn trim_samples<T>(samples: &mut VecDeque<(Instant, T)>, now: Instant) {
    while let Some((ts, _)) = samples.front() {
        if now.saturating_duration_since(*ts) > METRICS_WINDOW {
            samples.pop_front();
        } else {
            break;
        }
    }
}

async fn metrics_worker(
    mut rx: mpsc::Receiver<MetricsEvent>,
    limiter: Arc<AdaptiveLimiter>,
    min_submit_interval_ms: Arc<AtomicU64>,
) {
    let mut txn_samples: VecDeque<(Instant, TxnLifecycleDurations)> = VecDeque::new();
    let mut block_samples: VecDeque<(Instant, (usize, Duration))> = VecDeque::new();
    let mut ticker = tokio::time::interval(METRICS_TICK);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            maybe_event = rx.recv() => {
                let Some(event) = maybe_event else {
                    info!("metrics channel closed, stopping metrics worker");
                    break;
                };
                let now = Instant::now();
                match event {
                    MetricsEvent::TxnLifecycle {
                        txn_hash,
                        account,
                        durations,
                    } => {
                        debug!(
                            "txn {txn_hash} account {} lifecycle: queue {:?}, build {:?}, submit {:?}, confirm {:?}, total {:?}",
                            account,
                            durations.queue,
                            durations.build,
                            durations.submit,
                            durations.confirmation,
                            durations.total
                        );
                        txn_samples.push_back((now, durations));
                        trim_samples(&mut txn_samples, now);
                    }
                    MetricsEvent::BalanceTopUp { account, amount, duration } => {
                        info!(
                            "balance top-up for {} amount {} nano STC took {:?}",
                            account, amount, duration
                        );
                    }
                    MetricsEvent::BlockStats {
                        block_hash,
                        block_number,
                        txn_count,
                        interval,
                    } => {
                        debug!(
                            "block #{block_number} ({block_hash}) carried {txn_count} txns, interval {:?}",
                            interval
                        );
                        block_samples.push_back((now, (txn_count, interval)));
                        trim_samples(&mut block_samples, now);
                    }
                }
            }
            _ = ticker.tick() => {
                let now = Instant::now();
                trim_samples(&mut txn_samples, now);
                trim_samples(&mut block_samples, now);

                let avg_txn_per_block = if block_samples.is_empty() {
                    None
                } else {
                    let sum: f64 = block_samples
                        .iter()
                        .map(|(_, (count, _))| *count as f64)
                        .sum();
                    Some(sum / block_samples.len() as f64)
                };
                let avg_block_interval = if block_samples.is_empty() {
                    None
                } else {
                    let sum: f64 = block_samples
                        .iter()
                        .map(|(_, (_, interval))| interval.as_secs_f64())
                        .sum();
                    Some(sum / block_samples.len() as f64)
                };
                let avg_queue = if txn_samples.is_empty() {
                    None
                } else {
                    let sum: f64 = txn_samples
                        .iter()
                        .map(|(_, d)| d.queue.as_secs_f64())
                        .sum();
                    Some(Duration::from_secs_f64(sum / txn_samples.len() as f64))
                };
                let avg_build = if txn_samples.is_empty() {
                    None
                } else {
                    let sum: f64 = txn_samples
                        .iter()
                        .map(|(_, d)| d.build.as_secs_f64())
                        .sum();
                    Some(Duration::from_secs_f64(sum / txn_samples.len() as f64))
                };
                let avg_submit = if txn_samples.is_empty() {
                    None
                } else {
                    let sum: f64 = txn_samples
                        .iter()
                        .map(|(_, d)| d.submit.as_secs_f64())
                        .sum();
                    Some(Duration::from_secs_f64(sum / txn_samples.len() as f64))
                };
                let avg_confirmation = if txn_samples.is_empty() {
                    None
                } else {
                    let sum: f64 = txn_samples
                        .iter()
                        .map(|(_, d)| d.confirmation.as_secs_f64())
                        .sum();
                    Some(Duration::from_secs_f64(sum / txn_samples.len() as f64))
                };
                let avg_total = if txn_samples.is_empty() {
                    None
                } else {
                    let sum: f64 = txn_samples
                        .iter()
                        .map(|(_, d)| d.total.as_secs_f64())
                        .sum();
                    Some(Duration::from_secs_f64(sum / txn_samples.len() as f64))
                };

                let current_limit = limiter.limit();
                let mut target_limit = current_limit;
                if let Some(avg_txn) = avg_txn_per_block {
                    let target_low = TARGET_TXNS_PER_BLOCK * (1.0 - TXN_PER_BLOCK_TOLERANCE);
                    let target_high = TARGET_TXNS_PER_BLOCK * (1.0 + TXN_PER_BLOCK_TOLERANCE);
                    if avg_txn < target_low {
                        target_limit = (target_limit + CONCURRENCY_STEP).min(MAX_CONCURRENCY);
                    } else if avg_txn > target_high {
                        target_limit = target_limit.saturating_sub(CONCURRENCY_STEP).max(MIN_CONCURRENCY);
                    }
                }
                if let Some(avg_conf) = avg_confirmation {
                    if avg_conf > MAX_CONFIRMATION_LATENCY {
                        target_limit = target_limit.saturating_sub(CONCURRENCY_STEP).max(MIN_CONCURRENCY);
                    }
                }

                if target_limit != current_limit {
                    limiter.update_limit(target_limit).await;
                    info!(
                        "Adjusting active submissions from {} to {} (avg_txn/block: {:.1}, avg_confirm: {:?})",
                        current_limit,
                        target_limit,
                        avg_txn_per_block.unwrap_or(0.0),
                        avg_confirmation.unwrap_or_default()
                    );
                }

                let current_interval_ms = min_submit_interval_ms.load(Ordering::Relaxed);
                let mut new_interval_ms = current_interval_ms;
                if let Some(avg_conf) = avg_confirmation {
                    let half_confirmation =
                        Duration::from_secs_f64(MAX_CONFIRMATION_LATENCY.as_secs_f64() / 2.0);
                    if avg_conf > MAX_CONFIRMATION_LATENCY {
                        new_interval_ms = (current_interval_ms + MIN_INTERVAL_STEP_MS).min(MIN_INTERVAL_MAX_MS);
                    } else if avg_conf < half_confirmation {
                        new_interval_ms = current_interval_ms
                            .saturating_sub(MIN_INTERVAL_STEP_MS)
                            .max(MIN_INTERVAL_MIN_MS);
                    }
                }
                if new_interval_ms != current_interval_ms {
                    min_submit_interval_ms.store(new_interval_ms, Ordering::SeqCst);
                    info!(
                        "Updated min submit interval to {} ms (avg_confirm: {:?})",
                        new_interval_ms,
                        avg_confirmation.unwrap_or_default()
                    );
                }

                if let (Some(avg_txn), Some(avg_interval)) = (avg_txn_per_block, avg_block_interval) {
                    info!(
                        "Metrics window: avg_txn/block {:.1}, avg_block_interval {:.2}s, avg_queue {:?}, avg_build {:?}, avg_submit {:?}, avg_confirm {:?}, avg_total {:?}, active_limit {}, submit_interval {} ms",
                        avg_txn,
                        avg_interval,
                        avg_queue.unwrap_or_default(),
                        avg_build.unwrap_or_default(),
                        avg_submit.unwrap_or_default(),
                        avg_confirmation.unwrap_or_default(),
                        avg_total.unwrap_or_default(),
                        limiter.limit(),
                        min_submit_interval_ms.load(Ordering::Relaxed)
                    );
                }
            }
        }
    }
}
async fn info_worker(client: Arc<AsyncRpcClient>) {
    loop {
        if let Ok(node_info) = client.node_info().await {
            if let Some(guard) = GLOBAL_NODE_INFO.get() {
                let mut info = guard.write().await;
                *info = node_info;
            } else {
                warn!("Failed to get node info");
            }
        }
        sleep(Duration::from_secs(60)).await;
    }
}

pub async fn async_main(
    client: Arc<AsyncRpcClient>,
    target: AccountAddress,
    account_path: String,
) -> Result<()> {
    let min_balance: u128 = INITIAL_BALANCE;
    let funding = &*FUNDING_ACCOUNT;
    let node_info = client.node_info().await?;
    set_info(node_info);

    let mut accounts = load_accounts(&account_path).await?;
    accounts.shuffle(&mut rand::rng());

    let mut handles = Vec::new();
    let initial_limit = {
        let limit = std::cmp::min(accounts.len(), INITIAL_CONCURRENCY);
        let limit = std::cmp::max(limit, MIN_CONCURRENCY);
        limit.clamp(MIN_CONCURRENCY, MAX_CONCURRENCY)
    };
    let limiter = Arc::new(AdaptiveLimiter::new(initial_limit));
    let min_submit_interval_ms = Arc::new(AtomicU64::new(0));
    let (metrics_tx, metrics_rx) = mpsc::channel(4096);
    let metrics_handle = tokio::spawn({
        let limiter = Arc::clone(&limiter);
        let min_submit_interval_ms = Arc::clone(&min_submit_interval_ms);
        async move {
            metrics_worker(metrics_rx, limiter, min_submit_interval_ms).await;
        }
    });
    handles.push(metrics_handle);

    let info_handle = tokio::spawn({
        let client = Arc::clone(&client);
        async move {
            info_worker(client).await;
        }
    });
    handles.push(info_handle);

    let (tx, rx) = mpsc::channel(10240);
    let (confirm_tx, confirm_rx) = mpsc::channel(10240);

    let txn_confirmer = tokio::spawn({
        let client = Arc::clone(&client);
        let metrics_tx = metrics_tx.clone();
        async move {
            txn_confirmer(client, confirm_rx, metrics_tx).await;
        }
    });
    handles.push(txn_confirmer);

    for entry in accounts {
        let handle = tokio::spawn({
            let client = Arc::clone(&client);
            let tx_amount = DEFAULT_AMOUNT;
            let tx = tx.clone();
            let tx1 = confirm_tx.clone();
            let limiter = Arc::clone(&limiter);
            let min_submit_interval_ms = Arc::clone(&min_submit_interval_ms);
            let metrics_tx = metrics_tx.clone();
            async move {
                account_worker(
                    client,
                    entry,
                    target,
                    min_balance,
                    tx_amount,
                    tx,
                    tx1,
                    limiter,
                    min_submit_interval_ms,
                    metrics_tx,
                )
                .await;
            }
        });
        handles.push(handle);
    }
    let balancer = tokio::spawn({
        let client = Arc::clone(&client);
        let metrics_tx = metrics_tx.clone();
        async move {
            balancer_worker(client, funding, min_balance, rx, metrics_tx).await;
        }
    });
    handles.push(balancer);

    futures::future::join_all(handles).await;
    Ok(())
}
