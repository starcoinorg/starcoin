// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::BLOCK_RELAYER_METRICS;
use anyhow::Result;
use crypto::HashValue;
use futures::FutureExt;
use logger::prelude::*;
use starcoin_block_relayer_api::{NetCmpctBlockMessage, PeerCmpctBlockEvent};
use starcoin_network::NetworkAsyncService;
use starcoin_network_rpc_api::{gen_client::NetworkRpcClient, GetTxns};
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_sync::helper::get_txns;
use starcoin_sync_api::PeerNewBlock;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::SyncStatusChangeEvent;
use starcoin_types::{
    block::{Block, BlockBody},
    cmpact_block::{CompactBlock, PrefiledTxn, ShortId},
    peer_info::PeerId,
    system_events::NewHeadBlock,
    transaction::{SignedUserTransaction, Transaction},
};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

pub struct BlockRelayer {
    txpool: TxPoolService,
    sync_status: Option<SyncStatus>,
}

impl ServiceFactory<Self> for BlockRelayer {
    fn create(ctx: &mut ServiceContext<BlockRelayer>) -> Result<BlockRelayer> {
        let txpool = ctx.get_shared::<TxPoolService>()?;
        Ok(Self::new(txpool))
    }
}

impl BlockRelayer {
    pub fn new(txpool: TxPoolService) -> Self {
        Self {
            txpool,
            sync_status: None,
        }
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_synced(),
            None => false,
        }
    }

    async fn fill_compact_block(
        txpool: TxPoolService,
        rpc_client: NetworkRpcClient,
        compact_block: CompactBlock,
        peer_id: PeerId,
    ) -> Result<Block> {
        let txns_pool_vec = txpool.get_pending_txns(None, None);
        let txns_pool_map: HashMap<ShortId, &SignedUserTransaction> = {
            let pool_id_txn_iter = txns_pool_vec
                .iter()
                .map(|txn| (Transaction::UserTransaction(txn.clone()).id(), txn))
                .map(|(id, txn)| (ShortId(id), txn));
            HashMap::from_iter(pool_id_txn_iter)
        };
        let txns = {
            let mut txns: Vec<Option<SignedUserTransaction>> =
                vec![None; compact_block.short_ids.len()];
            let mut missing_txn_short_ids = HashSet::new();
            // Fill the block txns by tx pool
            for (index, short_id) in compact_block.short_ids.iter().enumerate() {
                if let Some(txn) = txns_pool_map.get(short_id) {
                    BLOCK_RELAYER_METRICS.txns_filled_from_txpool.inc();
                    txns[index] = Some((*txn).clone());
                } else {
                    missing_txn_short_ids.insert(short_id);
                }
            }

            // Fill the block txns by prefilled txn
            for prefilled_txn in compact_block.prefilled_txn {
                if prefilled_txn.index as usize >= txns.len() {
                    continue;
                }
                txns[prefilled_txn.index as usize] = Some(prefilled_txn.clone().tx);
                BLOCK_RELAYER_METRICS.txns_filled_from_prefill.inc();
                missing_txn_short_ids.remove(&ShortId(
                    Transaction::UserTransaction(prefilled_txn.tx).id(),
                ));
            }
            // Fetch the missing txns from peer
            let missing_txn_ids: Vec<HashValue> = missing_txn_short_ids
                .iter()
                .map(|&short_id| short_id.0)
                .collect();
            let fetched_missing_txn = get_txns(
                &rpc_client,
                peer_id,
                GetTxns {
                    ids: Some(missing_txn_ids),
                },
            )
            .await?
            .txns;
            let fetched_missing_txn_map: HashMap<ShortId, &SignedUserTransaction> = {
                let iter = fetched_missing_txn
                    .iter()
                    .map(|txn| (Transaction::UserTransaction(txn.clone()).id(), txn))
                    .map(|(id, txn)| (ShortId(id), txn));
                HashMap::from_iter(iter)
            };
            for (index, short_id) in compact_block.short_ids.iter().enumerate() {
                if txns[index].is_none() {
                    if let Some(&txn) = fetched_missing_txn_map.get(short_id) {
                        BLOCK_RELAYER_METRICS.txns_filled_from_network.inc();
                        txns[index] = Some(txn.clone());
                    }
                }
            }
            txns.iter()
                .filter(|&txn| txn.is_some())
                .map(|txn| txn.clone().unwrap())
                .collect()
        };
        let body = BlockBody::new(txns, compact_block.uncles);
        let block = Block::new(compact_block.header, body);
        Ok(block)
    }

    fn block_into_compact(&self, block: Block) -> CompactBlock {
        let mut prefilled_txn = Vec::new();
        let txns_pool_map: HashMap<HashValue, SignedUserTransaction> = {
            let pool_id_txn = self.txpool.get_pending_txns(None, None);
            HashMap::from_iter(
                pool_id_txn
                    .iter()
                    .map(|txn| (Transaction::UserTransaction(txn.clone()).id(), txn.clone())),
            )
        };
        for (index, txn) in block.transactions().iter().enumerate() {
            let id = Transaction::UserTransaction(txn.clone()).id();
            if !txns_pool_map.contains_key(&id) {
                prefilled_txn.push(PrefiledTxn {
                    index: index as u64,
                    tx: txn.clone(),
                });
            }
        }
        // TODO: prefill txns always equal to block.transactions.
        prefilled_txn.clear();
        CompactBlock::new(&block, prefilled_txn)
    }

    fn handle_block_event(
        &self,
        cmpct_block_msg: PeerCmpctBlockEvent,
        ctx: &mut ServiceContext<BlockRelayer>,
    ) -> Result<()> {
        let network = ctx.get_shared::<NetworkAsyncService>()?;
        let block_connector_service = ctx.service_ref::<BlockConnectorService>()?.clone();
        //TODO use VerifiedRpcClient and filter peers, avoid fetch from a no synced peer.
        let rpc_client = NetworkRpcClient::new(network);
        let txpool = self.txpool.clone();
        let fut = async move {
            let compact_block = cmpct_block_msg.compact_block;
            let peer_id = cmpct_block_msg.peer_id;
            debug!("Receive peer compact block event from peer id:{}", peer_id);
            let block = BlockRelayer::fill_compact_block(
                txpool,
                rpc_client,
                compact_block,
                peer_id.clone(),
            )
            .await?;
            block_connector_service.notify(PeerNewBlock::new(peer_id, block))?;
            Ok(())
        };
        ctx.spawn(fut.then(|result: Result<()>| async move {
            if let Err(e) = result {
                error!("[block-relay] process PeerCmpctBlockEvent error {:?}", e);
            }
        }));
        Ok(())
    }
}

impl ActorService for BlockRelayer {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for BlockRelayer {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, _ctx: &mut ServiceContext<Self>) {
        self.sync_status = Some(msg.0);
    }
}

impl EventHandler<Self, NewHeadBlock> for BlockRelayer {
    fn handle_event(&mut self, event: NewHeadBlock, ctx: &mut ServiceContext<BlockRelayer>) {
        debug!(
            "[block-relay] Handle new head block event, block_id: {:?}",
            event.0.get_block().id()
        );
        if !self.is_synced() {
            debug!("[block-relay] Ignore NewHeadBlock event because the node has not been synchronized yet.");
            return;
        }
        let compact_block = self.block_into_compact(event.0.get_block().clone());
        let total_difficulty = event.0.get_total_difficulty();
        let net_cmpct_block_msg = NetCmpctBlockMessage {
            compact_block,
            total_difficulty,
        };
        //TODO directly send to network.
        ctx.broadcast(net_cmpct_block_msg);
    }
}

impl EventHandler<Self, PeerCmpctBlockEvent> for BlockRelayer {
    fn handle_event(
        &mut self,
        cmpct_block_msg: PeerCmpctBlockEvent,
        ctx: &mut ServiceContext<BlockRelayer>,
    ) {
        if !self.is_synced() {
            debug!("[block-relay] Ignore PeerCmpctBlock event because the node has not been synchronized yet.");
            return;
        }
        //TODO filter by total_difficulty and block number, ignore too old block.
        if let Err(e) = self.handle_block_event(cmpct_block_msg, ctx) {
            error!("[block-relay] handle PeerCmpctBlock event error: {:?}", e);
        }
    }
}
