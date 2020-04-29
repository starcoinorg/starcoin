// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
#[macro_use]
extern crate async_trait;
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate trace_time;
#[macro_use]
extern crate prometheus;
extern crate transaction_pool as tx_pool;

use crate::counters::TXPOOL_SERVICE_HISTOGRAM;
pub use crate::pool::TxStatus;
use crate::tx_pool_service_impl::{
    ChainNewBlock, GetPendingTxns, ImportTxns, NextSequenceNumber, RemoveTxn, SubscribeTxns,
    TxPoolActor,
};
use actix::prelude::*;
use anyhow::Result;
use common_crypto::hash::HashValue;
use futures_channel::mpsc;
use starcoin_bus::BusActor;
use starcoin_config::TxPoolConfig;
use starcoin_txpool_api::TxPoolAsyncService;
use std::{fmt::Debug, sync::Arc};
use storage::Store;
#[cfg(test)]
use types::block::BlockHeader;
use types::{account_address::AccountAddress, transaction, transaction::SignedUserTransaction};

mod counters;
mod pool;
mod pool_client;
#[cfg(test)]
mod test;
mod tx_pool_service_impl;

#[derive(Clone, Debug)]
pub struct TxPoolRef {
    addr: actix::Addr<TxPoolActor>,
}

impl TxPoolRef {
    pub fn start(
        pool_config: TxPoolConfig,
        storage: Arc<dyn Store>,
        best_block_hash: HashValue,
        bus: actix::Addr<BusActor>,
    ) -> TxPoolRef {
        let best_block = match storage.get_block_by_hash(best_block_hash) {
            Err(e) => panic!("fail to read storage, {}", e),
            Ok(None) => panic!(
                "best block id {} should exists in storage",
                &best_block_hash
            ),
            Ok(Some(block)) => block,
        };
        let best_block_header = best_block.into_inner().0;
        let pool = TxPoolActor::new(pool_config, storage, best_block_header, bus);
        let pool_addr = pool.start();
        TxPoolRef { addr: pool_addr }
    }

    #[cfg(test)]
    pub fn start_with_best_block_header(
        storage: Arc<dyn Store>,
        best_block_header: BlockHeader,
        bus: actix::Addr<BusActor>,
    ) -> TxPoolRef {
        let pool = TxPoolActor::new(TxPoolConfig::default(), storage, best_block_header, bus);
        let pool_addr = pool.start();
        TxPoolRef { addr: pool_addr }
    }
}

#[async_trait]
impl TxPoolAsyncService for TxPoolRef {
    async fn add(self, txn: SignedUserTransaction) -> Result<bool> {
        let mut result = self.add_txns(vec![txn]).await?;
        Ok(result.pop().unwrap().is_ok())
    }

    async fn add_txns(
        self,
        txns: Vec<SignedUserTransaction>,
    ) -> Result<Vec<Result<(), transaction::TransactionError>>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["add_txns"])
            .start_timer();
        let result = self.addr.send(ImportTxns { txns }).await;
        timer.observe_duration();
        match result {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r),
        }
    }

    async fn remove_txn(
        self,
        txn_hash: HashValue,
        is_invalid: bool,
    ) -> Result<Option<SignedUserTransaction>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["remove_txn"])
            .start_timer();
        let result = self
            .addr
            .send(RemoveTxn {
                txn_hash,
                is_invalid,
            })
            .await;
        timer.observe_duration();
        match result {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r.map(|v| v.signed().clone())),
        }
    }

    async fn get_pending_txns(self, max_len: Option<u64>) -> Result<Vec<SignedUserTransaction>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["get_pending_txns"])
            .start_timer();
        let result = self
            .addr
            .send(GetPendingTxns {
                max_len: max_len.unwrap_or(u64::max_value()),
            })
            .await;
        timer.observe_duration();
        match result {
            Ok(r) => Ok(r.into_iter().map(|t| t.signed().clone()).collect()),
            Err(e) => Err(e.into()),
        }
    }
    async fn next_sequence_number(self, address: AccountAddress) -> Result<Option<u64>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["next_sequence_number"])
            .start_timer();
        let result = self.addr.send(NextSequenceNumber { address }).await;
        timer.observe_duration();
        result.map_err(|e| e.into())
    }

    async fn subscribe_txns(
        self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["subscribe_txns"])
            .start_timer();
        let result = self.addr.send(SubscribeTxns).await;
        timer.observe_duration();
        match result {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r),
        }
    }

    /// when new block happened in chain, use this to notify txn pool
    /// the `HashValue` of `enacted`/`retracted` is the hash of txns.
    /// enacted: the txns which enter into main chain.
    /// retracted: the txns which is rollbacked.
    async fn rollback(
        self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()> {
        let timer = TXPOOL_SERVICE_HISTOGRAM
            .with_label_values(&["rollback"])
            .start_timer();
        let result = self.addr.send(ChainNewBlock { enacted, retracted }).await;
        timer.observe_duration();
        match result {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r?),
        }
    }
}
