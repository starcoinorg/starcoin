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
extern crate transaction_pool as tx_pool;

pub use crate::pool::TxStatus;
use crate::tx_pool_service_impl::{
    ChainNewBlock, GetPendingTxns, ImportTxns, RemoveTxn, SubscribeTxns, TxPoolActor,
};
use actix::prelude::*;
use anyhow::Result;
use common_crypto::hash::HashValue;
use futures_channel::mpsc;
use starcoin_bus::BusActor;
use starcoin_config::TxPoolConfig;
use starcoin_txpool_api::TxPoolAsyncService;
use std::{fmt::Debug, sync::Arc};
use storage::BlockStore;
use storage::Storage;
#[cfg(test)]
use types::block::BlockHeader;
use types::{block::Block, transaction, transaction::SignedUserTransaction};
mod pool;
mod pool_client;
#[cfg(test)]
mod test;
mod tx_pool_service_impl;

trait BlockReader {
    fn get_block_by_hash(&self, block_hash: HashValue) -> Result<Option<Block>>;
}

#[derive(Clone, Debug)]
pub struct TxPoolRef {
    addr: actix::Addr<TxPoolActor>,
}

impl TxPoolRef {
    pub fn start(
        pool_config: TxPoolConfig,
        storage: Arc<Storage>,
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
        storage: Arc<Storage>,
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
        let request = self.addr.send(ImportTxns { txns });

        match request.await {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r),
        }
    }

    async fn remove_txn(
        self,
        txn_hash: HashValue,
        is_invalid: bool,
    ) -> Result<Option<SignedUserTransaction>> {
        match self
            .addr
            .send(RemoveTxn {
                txn_hash,
                is_invalid,
            })
            .await
        {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r.map(|v| v.signed().clone())),
        }
    }

    async fn get_pending_txns(self, max_len: Option<u64>) -> Result<Vec<SignedUserTransaction>> {
        match self
            .addr
            .send(GetPendingTxns {
                max_len: max_len.unwrap_or_else(|| u64::max_value()),
            })
            .await
        {
            Ok(r) => Ok(r.into_iter().map(|t| t.signed().clone()).collect()),
            Err(e) => Err(e.into()),
        }
    }

    async fn subscribe_txns(
        self,
    ) -> Result<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>> {
        match self.addr.send(SubscribeTxns).await {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r),
        }
    }

    /// when new block happened in chain, use this to notify txn pool
    /// the `HashValue` of `enacted`/`retracted` is the hash of blocks.
    /// enacted: the blocks which enter into main chain.
    /// retracted: the blocks which is rollbacked.
    async fn chain_new_blocks(
        self,
        _enacted: Vec<HashValue>,
        _retracted: Vec<HashValue>,
    ) -> Result<()> {
        todo!()
    }

    async fn rollback(
        self,
        enacted: Vec<SignedUserTransaction>,
        retracted: Vec<SignedUserTransaction>,
    ) -> Result<()> {
        match self.addr.send(ChainNewBlock { enacted, retracted }).await {
            Err(e) => Err(e.into()),
            Ok(r) => Ok(r?),
        }
    }
}

#[derive(Debug, Clone)]
struct NoneBlockReader;
impl BlockReader for NoneBlockReader {
    fn get_block_by_hash(&self, _block_hash: HashValue) -> Result<Option<Block>> {
        Ok(None)
    }
}

struct StorageBlockReader {
    storage: Arc<Storage>,
}

/// TODO: enhance me when storage impl Debug
impl Debug for StorageBlockReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "storage block reader")
    }
}

impl Clone for StorageBlockReader {
    fn clone(&self) -> Self {
        StorageBlockReader {
            storage: self.storage.clone(),
        }
    }
}

impl BlockReader for StorageBlockReader {
    fn get_block_by_hash(&self, block_hash: HashValue) -> Result<Option<Block>> {
        self.storage.get_block_by_hash(block_hash)
    }
}
