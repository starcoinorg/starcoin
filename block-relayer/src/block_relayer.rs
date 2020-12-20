// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::BLOCK_RELAYER_METRICS;
use anyhow::Result;
use crypto::HashValue;
use futures::FutureExt;
use logger::prelude::*;
use network_api::messages::{CompactBlockMessage, NotificationMessage, PeerCompactBlockMessage};
use network_api::NetworkService;
use starcoin_network::NetworkServiceRef;
use starcoin_network_rpc_api::{gen_client::NetworkRpcClient, GetTxnsWithHash};
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_sync::helper::get_txns_with_hash;
use starcoin_sync_api::PeerNewBlock;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::SyncStatusChangeEvent;
use starcoin_types::{
    block::{Block, BlockBody},
    cmpact_block::{CompactBlock, ShortId},
    peer_info::PeerId,
    system_events::NewHeadBlock,
    transaction::{SignedUserTransaction, Transaction},
};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
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
        let txns = {
            let mut txns: Vec<Option<SignedUserTransaction>> =
                vec![None; compact_block.short_ids.len()];
            let mut missing_txn_short_ids = HashSet::new();
            // Fill the block txns by tx pool
            for (index, short_id) in compact_block.short_ids.iter().enumerate() {
                if let Some(txn) = txpool.find_txn(&short_id.0) {
                    BLOCK_RELAYER_METRICS.txns_filled_from_txpool.inc();
                    txns[index] = Some(txn);
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
            let (_, fetched_missing_txn) = get_txns_with_hash(
                &rpc_client,
                peer_id,
                GetTxnsWithHash {
                    ids: missing_txn_ids,
                },
            )
            .await?;
            let mut fetched_missing_txn_map: HashMap<ShortId, Result<SignedUserTransaction>> = {
                let iter = fetched_missing_txn
                    .into_iter()
                    .map(|data| (ShortId(data.id()), data.try_into()));
                HashMap::from_iter(iter)
            };
            for (index, short_id) in compact_block.short_ids.iter().enumerate() {
                if txns[index].is_none() {
                    if let Some(txn) = fetched_missing_txn_map.remove(short_id) {
                        txns[index] = Some(txn?);
                        BLOCK_RELAYER_METRICS.txns_filled_from_network.inc();
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

    fn handle_block_event(
        &self,
        compact_block_msg: PeerCompactBlockMessage,
        ctx: &mut ServiceContext<BlockRelayer>,
    ) -> Result<()> {
        let network = ctx.get_shared::<NetworkServiceRef>()?;
        let block_connector_service = ctx.service_ref::<BlockConnectorService>()?.clone();
        let rpc_client = NetworkRpcClient::new(network);
        let txpool = self.txpool.clone();
        let fut = async move {
            let compact_block = compact_block_msg.message.compact_block;
            let peer_id = compact_block_msg.peer_id;
            debug!("Receive peer compact block event from peer id:{}", peer_id);
            let block_id = compact_block.header.id();
            if let Ok(Some(_)) = txpool.get_store().get_failed_block_by_id(block_id) {
                debug!("Block is failed block : {:?}", block_id);
            } else {
                let block = BlockRelayer::fill_compact_block(
                    txpool.clone(),
                    rpc_client,
                    compact_block,
                    peer_id.clone(),
                )
                .await?;
                block_connector_service.notify(PeerNewBlock::new(peer_id, block))?;
            }
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
            event.0.block().id()
        );
        if !self.is_synced() {
            debug!("[block-relay] Ignore NewHeadBlock event because the node has not been synchronized yet.");
            return;
        }
        let network = match ctx.get_shared::<NetworkServiceRef>() {
            Ok(network) => network,
            Err(e) => {
                error!("Get network service error: {:?}", e);
                return;
            }
        };
        let compact_block = event.0.block().clone().into();
        let total_difficulty = event.0.total_difficulty();
        let compact_block_msg = CompactBlockMessage {
            compact_block,
            total_difficulty,
        };
        network.broadcast(NotificationMessage::CompactBlock(Box::new(
            compact_block_msg,
        )));
    }
}

impl EventHandler<Self, PeerCompactBlockMessage> for BlockRelayer {
    fn handle_event(
        &mut self,
        compact_block_msg: PeerCompactBlockMessage,
        ctx: &mut ServiceContext<BlockRelayer>,
    ) {
        if !self.is_synced() {
            debug!("[block-relay] Ignore PeerCompactBlockMessage because the node has not been synchronized yet.");
            return;
        }
        let sync_status = self
            .sync_status
            .as_ref()
            .expect("Sync status should bean some at here");
        let current_total_difficulty = sync_status.chain_status().total_difficulty();
        let block_total_difficulty = compact_block_msg.message.total_difficulty;
        let block_id = compact_block_msg.message.compact_block.header.id();
        if current_total_difficulty > block_total_difficulty {
            debug!("[block-relay] Ignore PeerCompactBlockMessage because node current total_difficulty({}) > block({})'s total_difficulty({}).", current_total_difficulty, block_id, block_total_difficulty);
            return;
        }
        if let Err(e) = self.handle_block_event(compact_block_msg, ctx) {
            error!(
                "[block-relay] handle PeerCompactBlockMessage error: {:?}",
                e
            );
        }
    }
}
