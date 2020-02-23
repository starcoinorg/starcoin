// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::pool::{
    replace::ReplaceByScoreAndReadiness, scoring::NonceAndGasPrice, verifier, NonceClient,
    PrioritizationStrategy, VerifierOptions,
};
use crate::{
    pool,
    pool::{PendingOrdering, PendingSettings, VerifiedTransaction},
    GetPendingTransactions,
};
use actix::{
    dev::{MessageResponse, ResponseChannel},
    fut::wrap_future,
    prelude::*,
};
use actix_rt;
use anyhow::Result;
use futures::lock::Mutex as FutureMutux;
use pool::Gas;
use std::sync::Arc;
use types::{
    transaction,
    transaction::{SignatureCheckedTransaction, SignedUserTransaction, UnverifiedUserTransaction},
};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TxPool<C>
where
    C: NonceClient + Clone,
{
    addr: Addr<TxPoolActor<C>>,
}

impl<C> TxPool<C>
where
    C: NonceClient + Clone,
{
    pub fn start(client: C) -> TxPool<C>
    where
        C: NonceClient + Clone,
    {
        let addr = TxPoolActor::new(client).start();
        TxPool { addr }
    }

    pub async fn import_txns(
        &self,
        txns: Vec<transaction::SignedUserTransaction>,
    ) -> Result<Vec<Result<(), transaction::TransactionError>>> {
        let r = self.addr.send(ImportTxns { txns });
        Ok(r.await?)
    }

    // pub async fn get_pending_txns(
    //     &self,
    //     best_block_number: u64,
    //     best_block_timestamp: u64,
    //     max_len: u64,
    // ) -> Option<Vec<Arc<pool::VerifiedTransaction>>> {
    //     let r = self.addr.send(GetPendingTxns {
    //         best_block_number,
    //         best_block_timestamp,
    //         max_len,
    //     });
    //     Ok(r.await?)
    // }
}

struct ImportTxns {
    txns: Vec<transaction::SignedUserTransaction>,
}

impl Message for ImportTxns {
    type Result = Vec<Result<(), transaction::TransactionError>>;
}

struct GetPendingTxns {
    best_block_number: u64,
    best_block_timestamp: u64,
    max_len: u64,
}

impl Message for GetPendingTxns {
    type Result = Option<Vec<Arc<pool::VerifiedTransaction>>>;
}

type Listener = tx_pool::NoopListener;
type Pool = tx_pool::Pool<VerifiedTransaction, NonceAndGasPrice, Listener>;
type TxnQueue = pool::TransactionQueue;
#[derive(Debug)]
struct TxPoolActor<C>
where
    C: NonceClient + Clone,
{
    queue: Arc<FutureMutux<TxnQueue>>,
    nonce_client: C,
}

impl<C> TxPoolActor<C>
where
    C: NonceClient + Clone,
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
            nonce_client: client,
        }
    }
}

impl<C> Actor for TxPoolActor<C>
where
    C: NonceClient + Clone + Unpin,
{
    type Context = Context<Self>;
}

impl<C> Handler<ImportTxns> for TxPoolActor<C>
where
    C: NonceClient + Clone,
{
    type Result = ImportTxnsResponse<C>;

    fn handle(&mut self, msg: ImportTxns, ctx: &mut Self::Context) -> Self::Result {
        ImportTxnsResponse {
            msg,
            queue: self.queue.clone(),
            nonce_client: self.nonce_client.clone(),
        }
    }
}

struct ImportTxnsResponse<C>
where
    C: NonceClient + Clone,
{
    msg: ImportTxns,
    queue: Arc<FutureMutux<TxnQueue>>,
    nonce_client: C,
}

impl<C> MessageResponse<TxPoolActor<C>, ImportTxns> for ImportTxnsResponse<C>
where
    C: NonceClient + Clone + Unpin + 'static,
{
    fn handle<R: ResponseChannel<ImportTxns>>(
        self,
        ctx: &mut <TxPoolActor<C> as Actor>::Context,
        tx: Option<R>,
    ) {
        let Self {
            msg,
            queue,
            nonce_client,
        } = self;
        let ImportTxns { mut txns } = msg;
        ctx.wait(wrap_future(async move {
            let txns = txns
                .into_iter()
                .map(|t| verifier::Transaction::Unverified(t));

            let import_result = {
                let mut queue = queue.lock().await;
                queue.import(nonce_client, txns).await
            };
            if let Some(tx) = tx {
                tx.send(import_result)
            }
        }));
    }
}
