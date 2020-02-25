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
use futures::lock::Mutex as FutureMutux;
use pool::Gas;
use pool::SeqNumber;
use std::sync::Arc;
use storage::StarcoinStorage;
use traits::TxPoolAsyncService;
use types::account_address::AccountAddress;
use types::transaction;
use types::transaction::SignedUserTransaction;
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

#[async_trait]
impl AccountSeqNumberClient for CachedSeqNumberClient {
    async fn account_seq_number(&self, address: &AccountAddress) -> SeqNumber {
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
    queue: Arc<FutureMutux<TxnQueue>>,
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
        let queue = Arc::new(FutureMutux::new(queue));
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
    type Result = ImportTxnsResponse<C>;

    fn handle(&mut self, msg: ImportTxns, _ctx: &mut Self::Context) -> Self::Result {
        ImportTxnsResponse {
            msg,
            queue: self.queue.clone(),
            seq_number_client: self.seq_number_client.clone(),
        }
    }
}

struct ImportTxnsResponse<C>
where
    C: AccountSeqNumberClient,
{
    msg: ImportTxns,
    queue: Arc<FutureMutux<TxnQueue>>,
    seq_number_client: C,
}

impl<C> MessageResponse<TxPoolActor<C>, ImportTxns> for ImportTxnsResponse<C>
where
    C: AccountSeqNumberClient,
{
    fn handle<R: ResponseChannel<ImportTxns>>(
        self,
        ctx: &mut <TxPoolActor<C> as Actor>::Context,
        tx: Option<R>,
    ) {
        let Self {
            msg,
            queue,
            seq_number_client,
        } = self;
        let ImportTxns { txns } = msg;
        ctx.wait(wrap_future(async move {
            let txns = txns
                .into_iter()
                .map(|t| verifier::Transaction::Unverified(t));

            let import_result = {
                let mut queue = queue.lock().await;
                queue.import(seq_number_client, txns).await
            };
            if let Some(tx) = tx {
                tx.send(import_result)
            }
        }));
    }
}

struct GetPendingTxns {
    max_len: u64,
}

impl Message for GetPendingTxns {
    type Result = Vec<Arc<pool::VerifiedTransaction>>;
}

struct GetPendingTxnsResponse<C>
where
    C: AccountSeqNumberClient,
{
    msg: GetPendingTxns,
    queue: Arc<FutureMutux<TxnQueue>>,
    seq_number_client: C,
}
impl<C> MessageResponse<TxPoolActor<C>, GetPendingTxns> for GetPendingTxnsResponse<C>
where
    C: AccountSeqNumberClient,
{
    fn handle<R: ResponseChannel<GetPendingTxns>>(
        self,
        ctx: &mut <TxPoolActor<C> as Actor>::Context,
        tx: Option<R>,
    ) {
        let Self {
            msg,
            queue,
            seq_number_client,
        } = self;
        let GetPendingTxns { max_len } = msg;
        ctx.wait(wrap_future(async move {
            let import_result = {
                let mut queue = queue.lock().await;
                let pending_settings = PendingSettings {
                    block_number: u64::max_value(),
                    current_timestamp: u64::max_value(),
                    nonce_cap: None,
                    max_len: max_len as usize,
                    ordering: PendingOrdering::Priority,
                };
                queue.pending(seq_number_client, pending_settings).await
            };
            if let Some(tx) = tx {
                tx.send(import_result)
            }
        }));
    }
}
impl<C> Handler<GetPendingTxns> for TxPoolActor<C>
where
    C: AccountSeqNumberClient,
{
    type Result = GetPendingTxnsResponse<C>;

    fn handle(&mut self, msg: GetPendingTxns, _ctx: &mut Self::Context) -> Self::Result {
        GetPendingTxnsResponse {
            msg,
            queue: self.queue.clone(),
            seq_number_client: self.seq_number_client.clone(),
        }
    }
}
