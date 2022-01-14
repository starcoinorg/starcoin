// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::BlockRelayerMetrics;
use anyhow::{ensure, format_err, Result};
use config::NodeConfig;
use config::CRATE_VERSION;
use crypto::HashValue;
use futures::FutureExt;
use logger::prelude::*;
use network_api::messages::{CompactBlockMessage, NotificationMessage, PeerCompactBlockMessage};
use network_api::{NetworkService, PeerProvider, PeerSelector, PeerStrategy};
use starcoin_chain::verifier::StaticVerifier;
use starcoin_network::NetworkServiceRef;
use starcoin_network_rpc_api::GetTxnsWithHash;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_sync::block_connector::BlockConnectorService;
use starcoin_sync::verified_rpc_client::VerifiedRpcClient;
use starcoin_sync_api::PeerNewBlock;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::block::ExecutedBlock;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::{NewBranch, SyncStatusChangeEvent};
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
    metrics: Option<BlockRelayerMetrics>,
}

impl ServiceFactory<Self> for BlockRelayer {
    fn create(ctx: &mut ServiceContext<BlockRelayer>) -> Result<BlockRelayer> {
        let txpool = ctx.get_shared::<TxPoolService>()?;
        let node_config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let time_service = node_config.net().time_service();
        let metrics = node_config
            .metrics
            .registry()
            .and_then(|registry| BlockRelayerMetrics::register(registry).ok());
        Ok(Self::new(txpool, time_service, metrics))
    }
}

impl BlockRelayer {
    pub fn new(
        txpool: TxPoolService,
        time_service: Arc<dyn TimeService>,
        metrics: Option<BlockRelayerMetrics>,
    ) -> Self {
        Self {
            txpool,
            sync_status: None,
            time_service,
            metrics,
        }
    }

    pub fn is_nearly_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_nearly_synced(),
            None => false,
        }
    }

    fn broadcast_compact_block(
        &self,
        network: NetworkServiceRef,
        executed_block: Arc<ExecutedBlock>,
    ) {
        if !self.is_nearly_synced() {
            debug!("[block-relay] Ignore NewHeadBlock event because the node has not been synchronized yet.");
            return;
        }
        let compact_block = executed_block.block().clone().into();
        let compact_block_msg =
            CompactBlockMessage::new(compact_block, executed_block.block_info.clone());
        network.broadcast(NotificationMessage::CompactBlock(Box::new(
            compact_block_msg,
        )));
    }

    async fn fill_compact_block(
        txpool: TxPoolService,
        rpc_client: VerifiedRpcClient,
        compact_block: CompactBlock,
        peer_id: PeerId,
        metrics: Option<BlockRelayerMetrics>,
    ) -> Result<Block> {
        let expect_txn_len = compact_block.txn_len();
        let mut filled_from_txpool: u64 = 0;
        let mut filled_from_prefilled: u64 = 0;
        let mut filled_from_network: u64 = 0;

        let txns = if expect_txn_len == 0 {
            vec![]
        } else {
            let mut txns: Vec<Option<SignedUserTransaction>> = vec![None; expect_txn_len];

            let mut missing_txn_short_ids = HashSet::new();
            // Fill the block txns by tx pool
            for (index, short_id) in compact_block.short_ids.iter().enumerate() {
                if let Some(txn) = txpool.find_txn(&short_id.0) {
                    filled_from_txpool += 1;
                    txns[index] = Some(txn);
                } else {
                    missing_txn_short_ids.insert(short_id);
                }
            }

            //TODO move prefilled before txpool

            // Fill the block txns by prefilled txn
            for prefilled_txn in compact_block.prefilled_txn {
                if prefilled_txn.index as usize >= txns.len() {
                    continue;
                }
                let id = prefilled_txn.tx.id();
                txns[prefilled_txn.index as usize] = Some(prefilled_txn.tx);
                filled_from_prefilled += 1;
                missing_txn_short_ids.remove(&ShortId(id));
            }
            // Fetch the missing txns from peer
            let missing_txn_ids: Vec<HashValue> = missing_txn_short_ids
                .iter()
                .map(|&short_id| short_id.0)
                .collect();
            let mut fetched_missing_txn_map: HashMap<ShortId, Result<SignedUserTransaction>> =
                if missing_txn_ids.is_empty() {
                    HashMap::new()
                } else {
                    let (_, fetched_missing_txn) = rpc_client
                        .get_txns(
                            Some(peer_id),
                            GetTxnsWithHash {
                                ids: missing_txn_ids,
                            },
                        )
                        .await?;
                    fetched_missing_txn
                        .into_iter()
                        .map(|data| (ShortId(data.id()), data.try_into()))
                        .collect()
                };
            for (index, short_id) in compact_block.short_ids.iter().enumerate() {
                if txns[index].is_none() {
                    if let Some(txn) = fetched_missing_txn_map.remove(short_id) {
                        txns[index] = Some(txn?);
                        filled_from_network += 1;
                    }
                }
            }
            let collect_txns = txns.into_iter().flatten().collect::<Vec<_>>();
            ensure!(
                collect_txns.len() == expect_txn_len,
                "Fill compact block error, expect txn len: {}, but collect txn len: {}",
                collect_txns.len(),
                expect_txn_len
            );

            if let Some(metrics) = metrics {
                metrics
                    .txns_filled_total
                    .with_label_values(&["expect"])
                    .inc_by(expect_txn_len as u64);
                metrics
                    .txns_filled_total
                    .with_label_values(&["txpool"])
                    .inc_by(filled_from_txpool);
                metrics
                    .txns_filled_total
                    .with_label_values(&["network"])
                    .inc_by(filled_from_network);
                metrics
                    .txns_filled_total
                    .with_label_values(&["prefilled"])
                    .inc_by(filled_from_prefilled);
            }
            collect_txns
        };

        let body = BlockBody::new(txns, compact_block.uncles);
        let block = Block::new(compact_block.header, body);
        //ensure the block is filled correct.
        StaticVerifier::verify_body_hash(&block)?;
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
        let metrics = self.metrics.clone();
        let fut = async move {
            let compact_block = compact_block_msg.message.compact_block;
            let peer_id = compact_block_msg.peer_id;
            debug!("Receive peer compact block event from peer id:{}", peer_id);
            let block_id = compact_block.header.id();
            if let Ok(Some((_, _, _, version))) =
                txpool.get_store().get_failed_block_by_id(block_id)
            {
                if version == *CRATE_VERSION {
                    warn!("Block is failed block : {:?}", block_id);
                }
            } else {
                let peer = network.get_peer(peer_id.clone()).await?.ok_or_else(|| {
                    format_err!(
                        "CompatBlockMessage's peer {} is not connected",
                        peer_id.clone()
                    )
                })?;
                let peer_selector = PeerSelector::new(vec![peer], PeerStrategy::default(), None);
                let rpc_client = VerifiedRpcClient::new(peer_selector, network);
                let _timer = metrics
                    .as_ref()
                    .map(|metrics| metrics.txns_filled_time.start_timer());
                let block = BlockRelayer::fill_compact_block(
                    txpool.clone(),
                    rpc_client,
                    compact_block,
                    peer_id.clone(),
                    metrics,
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
        ctx.subscribe::<NewBranch>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        ctx.unsubscribe::<NewBranch>();
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
        let network = match ctx.get_shared::<NetworkServiceRef>() {
            Ok(network) => network,
            Err(e) => {
                error!("Get network service error: {:?}", e);
                return;
            }
        };
        self.broadcast_compact_block(network, event.0);
    }
}

impl EventHandler<Self, NewBranch> for BlockRelayer {
    fn handle_event(&mut self, event: NewBranch, ctx: &mut ServiceContext<BlockRelayer>) {
        debug!(
            "[block-relay] Handle new branch event, block_id: {:?}",
            event.0.block().id()
        );
        let network = match ctx.get_shared::<NetworkServiceRef>() {
            Ok(network) => network,
            Err(e) => {
                error!("Get network service error: {:?}", e);
                return;
            }
        };
        self.broadcast_compact_block(network, event.0);
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
        let time_sec: f64 = (time as f64) / 1000_f64;
        if let Some(metrics) = self.metrics.as_ref() {
            metrics.block_relay_time.observe(time_sec);
        }
        sl_info!(
            "{action} {hash} {time_sec}",
            time_sec = time_sec,
            hash = compact_block_msg.message.compact_block.header.id().to_hex(),
            action = "block_relay_time",
        );
        //TODO should filter too old block?

        if let Err(e) = self.handle_block_event(compact_block_msg, ctx) {
            if let Some(metrics) = self.metrics.as_ref() {
                metrics.txns_filled_failed_total.inc();
            }
            error!(
                "[block-relay] handle PeerCompactBlockMessage error: {:?}",
                e
            );
        }
    }
}
