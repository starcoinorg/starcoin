// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::BLOCK_RELAYER_METRICS;
use anyhow::Result;
use config::NodeConfig;
use crypto::HashValue;
use futures::FutureExt;
use logger::prelude::*;
use network_api::messages::{CompactBlockMessage, NotificationMessage, PeerCompactBlockMessage};
use network_api::{NetworkService, PeerProvider, PeerSelector, PeerStrategy};
use starcoin_network::NetworkServiceRef;
use starcoin_network_rpc_api::GetTxnsWithHash;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_sync::verified_rpc_client::VerifiedRpcClient;
use starcoin_sync_api::PeerNewBlock;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::SyncStatusChangeEvent;
use starcoin_types::time::TimeService;
use starcoin_types::{
    block::{Block, BlockBody},
    cmpact_block::{CompactBlock, ShortId},
    peer_info::PeerId,
    system_events::NewHeadBlock,
    transaction::SignedUserTransaction,
};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::sync::Arc;

pub struct BlockRelayer {
    txpool: TxPoolService,
    sync_status: Option<SyncStatus>,
    time_service: Arc<dyn TimeService>,
}

impl ServiceFactory<Self> for BlockRelayer {
    fn create(ctx: &mut ServiceContext<BlockRelayer>) -> Result<BlockRelayer> {
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let time_service = ctx.get_shared::<Arc<NodeConfig>>()?.net().time_service();
        Ok(Self::new(txpool, time_service))
    }
}

impl BlockRelayer {
    pub fn new(txpool: TxPoolService, time_service: Arc<dyn TimeService>) -> Self {
        Self {
            txpool,
            sync_status: None,
            time_service,
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
        rpc_client: VerifiedRpcClient,
        compact_block: CompactBlock,
        peer_id: PeerId,
    ) -> Result<Block> {
        let txns = {
            let mut txns: Vec<Option<SignedUserTransaction>> =
                vec![None; compact_block.short_ids.len()];
            BLOCK_RELAYER_METRICS
                .block_txns_count
                .set(compact_block.short_ids.len() as u64);
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
                let id = prefilled_txn.tx.id();
                txns[prefilled_txn.index as usize] = Some(prefilled_txn.tx);
                BLOCK_RELAYER_METRICS.txns_filled_from_prefill.inc();
                missing_txn_short_ids.remove(&ShortId(id));
            }
            // Fetch the missing txns from peer
            let missing_txn_ids: Vec<HashValue> = missing_txn_short_ids
                .iter()
                .map(|&short_id| short_id.0)
                .collect();
            let (_, fetched_missing_txn) = rpc_client
                .get_txns(
                    Some(peer_id),
                    GetTxnsWithHash {
                        ids: missing_txn_ids,
                    },
                )
                .await?;
            let mut fetched_missing_txn_map: HashMap<ShortId, Result<SignedUserTransaction>> = {
                fetched_missing_txn
                    .into_iter()
                    .map(|data| (ShortId(data.id()), data.try_into()))
                    .collect()
            };
            for (index, short_id) in compact_block.short_ids.iter().enumerate() {
                if txns[index].is_none() {
                    if let Some(txn) = fetched_missing_txn_map.remove(short_id) {
                        if txn.is_err() {
                            BLOCK_RELAYER_METRICS
                                .txns_filled_failed
                                .with_label_values(&["miss"])
                                .inc();
                        }
                        txns[index] = Some(txn?);
                        BLOCK_RELAYER_METRICS.txns_filled_from_network.inc();
                    }
                }
            }
            txns.into_iter().filter_map(|txn| txn).collect()
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
        let txpool = self.txpool.clone();
        let fut = async move {
            let compact_block = compact_block_msg.message.compact_block;
            let peer_id = compact_block_msg.peer_id;
            debug!("Receive peer compact block event from peer id:{}", peer_id);
            let block_id = compact_block.header.id();
            if let Ok(Some(_)) = txpool.get_store().get_failed_block_by_id(block_id) {
                warn!("Block is failed block : {:?}", block_id);
            } else {
                let peers = network.peer_set().await?;
                let peer_selector = PeerSelector::new(peers, PeerStrategy::default());
                let rpc_client = VerifiedRpcClient::new(peer_selector, network);
                let timer = BLOCK_RELAYER_METRICS.txns_filled_time.start_timer();
                let block = BlockRelayer::fill_compact_block(
                    txpool.clone(),
                    rpc_client,
                    compact_block,
                    peer_id.clone(),
                )
                .await?;
                timer.observe_duration();
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
        let compact_block_msg = CompactBlockMessage::new(compact_block, event.0.block_info.clone());
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
        let block_timestamp = compact_block_msg.message.compact_block.header.timestamp();
        let current_timestamp = self.time_service.now_millis();
        let time = current_timestamp.saturating_sub(block_timestamp);
        let time_sec = (time as f64) / 1000_f64;
        BLOCK_RELAYER_METRICS.block_broadcast_time.observe(time_sec);
        //TODO should filter too old block?

        if let Err(e) = self.handle_block_event(compact_block_msg, ctx) {
            BLOCK_RELAYER_METRICS
                .txns_filled_failed
                .with_label_values(&["error"])
                .inc();
            error!(
                "[block-relay] handle PeerCompactBlockMessage error: {:?}",
                e
            );
        }
    }
}
