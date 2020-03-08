// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::pool::{verifier, AccountSeqNumberClient, PrioritizationStrategy};
use crate::{
    pool,
    pool::{PendingOrdering, PendingSettings},
};
use actix::{
    dev::{MessageResponse, ResponseChannel},
    fut::wrap_future,
    prelude::*,
};
use anyhow::{Error, Result};
use common_crypto::hash::HashValue;

use futures_channel::mpsc;
use pool::{Gas, SeqNumber, TxStatus};
use std::sync::Arc;
use storage::StarcoinStorage;
use traits::TxPoolAsyncService;
use types::{account_address::AccountAddress, transaction, transaction::SignedUserTransaction};

pub type TxPoolRef = TxPool<CachedSeqNumberClient>;

#[derive(Clone, Debug)]
pub struct CachedSeqNumberClient {
    // storage: Arc<StarcoinStorage>,
}

impl CachedSeqNumberClient {
    pub fn new(_storage: Arc<StarcoinStorage>) -> Self {
        Self {}
    }
}

impl AccountSeqNumberClient for CachedSeqNumberClient {
    fn account_seq_number(&self, _address: &AccountAddress) -> SeqNumber {
        // TODO: hit real storage
        0u64
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TxPool<C>
where
    C: AccountSeqNumberClient,
{
    addr: Addr<TxPoolActor<C>>,
}

#[async_trait]
impl<C> TxPoolAsyncService for TxPool<C>
where
    C: AccountSeqNumberClient + Send,
{
    async fn add(self, txn: SignedUserTransaction) -> Result<bool> {
        match self.import_txns(vec![txn]).await {
            Err(e) => Err(Into::<Error>::into(e)),
            Ok(mut result) => Ok(result.pop().unwrap().is_ok()),
        }
    }

    async fn add_txns(
        self,
        txns: Vec<SignedUserTransaction>,
    ) -> Result<Vec<Result<(), transaction::TransactionError>>> {
        match self.import_txns(txns).await {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r),
        }
    }

    async fn get_pending_txns(self, max_len: Option<u64>) -> Result<Vec<SignedUserTransaction>> {
        match self
            .pending_txns(max_len.unwrap_or_else(|| u64::max_value()))
            .await
        {
            Ok(r) => Ok(r.into_iter().map(|t| t.signed().clone()).collect()),
            Err(e) => Err(e.into()),
        }
    }

    async fn subscribe_txns(
        self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>> {
        self.subscribe_txns_inner().await
    }

    async fn chain_new_blocks(
        self,
        enacted: Vec<HashValue>,
        retracted: Vec<HashValue>,
    ) -> Result<()> {
        self.chain_new_blocks_inner(enacted, retracted).await
    }
}

impl<C> TxPool<C>
where
    C: AccountSeqNumberClient,
{
    pub fn start(client: C) -> TxPool<C>
    where
        C: AccountSeqNumberClient,
    {
        let addr = TxPoolActor::new(client).start();
        TxPool { addr }
    }

    pub async fn import_txns(
        &self,
        txns: Vec<transaction::SignedUserTransaction>,
    ) -> Result<Vec<Result<(), transaction::TransactionError>>> {
        Ok(self.addr.send(ImportTxns { txns }).await?)
    }

    pub async fn pending_txns(&self, max_len: u64) -> Result<Vec<Arc<pool::VerifiedTransaction>>> {
        let r = self.addr.send(GetPendingTxns { max_len });
        Ok(r.await?)
    }

    pub async fn subscribe_txns_inner(
        &self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>> {
        let r = self.addr.send(SubscribeTxns);
        Ok(r.await?)
    }

    /// when new block happened in chain, use this to notify txn pool
    /// the `HashValue` of `enacted`/`retracted` is the hash of blocks.
    /// enacted: the blocks which enter into main chain.
    /// retracted: the blocks which is rollbacked.
    pub async fn chain_new_blocks_inner(
        &self,
        _enacted: Vec<HashValue>,
        _retracted: Vec<HashValue>,
    ) -> Result<()> {
        //TODO
        Ok(())
    }
}

struct ImportTxns {
    txns: Vec<transaction::SignedUserTransaction>,
}

impl Message for ImportTxns {
    type Result = Vec<Result<(), transaction::TransactionError>>;
}

type TxnQueue = pool::TransactionQueue;

#[derive(Debug)]
struct TxPoolActor<C>
where
    C: AccountSeqNumberClient,
{
    queue: TxnQueue,
    seq_number_client: C,
}

impl<C> TxPoolActor<C>
where
    C: AccountSeqNumberClient,
{
    pub fn new(client: C) -> Self {
        let verifier_options = pool::VerifierOptions {
            minimal_gas_price: 0,
            block_gas_limit: Gas::max_value(),
            tx_gas_limit: Gas::max_value(),
            no_early_reject: false,
        };
        let queue = TxnQueue::new(
            tx_pool::Options::default(),
            verifier_options,
            PrioritizationStrategy::GasPriceOnly,
        );
        // let queue = Arc::new(FutureMutux::new(queue));
        Self {
            queue,
            seq_number_client: client,
        }
    }
}

impl<C> Actor for TxPoolActor<C>
where
    C: AccountSeqNumberClient,
{
    type Context = Context<Self>;
}

impl<C> Handler<ImportTxns> for TxPoolActor<C>
where
    C: AccountSeqNumberClient,
{
    type Result = MessageResult<ImportTxns>;

    fn handle(&mut self, msg: ImportTxns, _ctx: &mut Self::Context) -> Self::Result {
        let ImportTxns { txns } = msg;

        let txns = txns
            .into_iter()
            .map(|t| verifier::Transaction::Unverified(t));

        let import_result = { self.queue.import(self.seq_number_client.clone(), txns) };
        MessageResult(import_result)
    }
}

struct GetPendingTxns {
    max_len: u64,
}

impl Message for GetPendingTxns {
    type Result = Vec<Arc<pool::VerifiedTransaction>>;
}

impl<C> Handler<GetPendingTxns> for TxPoolActor<C>
where
    C: AccountSeqNumberClient,
{
    type Result = MessageResult<GetPendingTxns>;

    fn handle(&mut self, msg: GetPendingTxns, _ctx: &mut Self::Context) -> Self::Result {
        let GetPendingTxns { max_len } = msg;
        let result = {
            let pending_settings = PendingSettings {
                block_number: u64::max_value(),
                current_timestamp: u64::max_value(),
                nonce_cap: None,
                max_len: max_len as usize,
                ordering: PendingOrdering::Priority,
            };
            self.queue
                .pending(self.seq_number_client.clone(), pending_settings)
        };
        MessageResult(result)
    }
}

pub struct SubscribeTxns;

impl Message for SubscribeTxns {
    type Result = mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>;
}

impl<C> Handler<SubscribeTxns> for TxPoolActor<C>
where
    C: AccountSeqNumberClient,
{
    type Result = MessageResult<SubscribeTxns>;

    fn handle(&mut self, _: SubscribeTxns, _ctx: &mut Self::Context) -> Self::Result {
        let result = {
            let (tx, rx) = mpsc::unbounded();
            self.queue.add_full_listener(tx);
            rx
        };
        MessageResult(result)
    }
}
