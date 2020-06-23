use crate::BlockRelayEvent;
use actix::{prelude::*, Actor, Addr, Context, Handler};
use anyhow::Result;
use bus::{Broadcast, BusActor, Subscription};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use network_api::NetworkService;
use starcoin_sync_api::PeerNewBlock;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    block::{Block, BlockBody, BlockDetail},
    cmpact_block::{CompactBlock, PrefiledTxn, ShortId},
    system_events::{self, PeerNewCmpctBlock},
    transaction::{SignedUserTransaction, Transaction},
    BLOCK_PROTOCOL_NAME,
};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::sync::Arc;
use traits::Consensus;

pub struct BlockRelayer<C, P, N>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    N: NetworkService + Sync + Send + 'static,
{
    _config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    txpool: P,
    network: N,
    phantom_c: PhantomData<C>,
}

impl<C, P, N> BlockRelayer<C, P, N>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    N: NetworkService + Sync + Send + 'static,
{
    pub fn new(
        _config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        txpool: P,
        network: N,
    ) -> Result<Addr<BlockRelayer<C, P, N>>> {
        let block_relayer = BlockRelayer {
            _config,
            bus,
            txpool,
            network,
            phantom_c: PhantomData,
        };
        Ok(block_relayer.start())
    }

    fn fill_compact_block(&self, compact_block: CompactBlock) -> Result<Block> {
        let txns_pool_vec = self.txpool.get_pending_txns(None);
        let txns_pool_map: HashMap<ShortId, &SignedUserTransaction> = {
            let pool_id_txn_iter = txns_pool_vec
                .iter()
                .map(|txn| (Transaction::UserTransaction(txn.clone()).id(), txn))
                .map(|(id, txn)| (ShortId(id), txn));
            HashMap::from_iter(pool_id_txn_iter)
        };
        let txns = {
            let mut txns: Vec<SignedUserTransaction> =
                Vec::with_capacity(compact_block.short_ids.len());
            for (index, short_id) in compact_block.short_ids.iter().enumerate() {
                if let Some(txn) = txns_pool_map.get(short_id) {
                    txns[index] = txn.clone().clone();
                };
            }
            for prefilled_txn in compact_block.prefilled_txn {
                if prefilled_txn.index as usize >= txns.len() {
                    continue;
                }
                txns[prefilled_txn.index as usize] = prefilled_txn.tx;
            }
            txns
        };
        let body = BlockBody::new(txns);
        let block = Block::new(compact_block.header, body);
        Ok(block)
    }

    fn broadcast(&self, new_head_block: system_events::NewHeadBlock) {
        let bus = self.bus.clone();
        let network = self.network.clone();
        Arbiter::spawn(async move {
            // TODO: CHAIN_METRICS.broadcast_head_count.inc();
            bus.do_send(Broadcast {
                msg: new_head_block.clone(),
            });
            if let Err(err) = network
                .broadcast_new_head_block(BLOCK_PROTOCOL_NAME.into(), new_head_block.clone())
                .await
            {
                let block_id = new_head_block.0.header().id();
                error!(
                    "Failed to broadcast new head block {:?} with error: {:?}",
                    block_id, err
                );
            }
        });
    }

    fn block_into_compact(&self, block: Block) -> CompactBlock {
        let mut prefilled_txn = Vec::new();
        let txns_pool_map: HashMap<HashValue, SignedUserTransaction> = {
            let pool_id_txn = self.txpool.get_pending_txns(None);
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
        CompactBlock::new(&block, prefilled_txn)
    }
}

impl<C, P, N> Actor for BlockRelayer<C, P, N>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    N: NetworkService + Sync + Send + 'static,
{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let cmpctblock_recipient = ctx.address().recipient::<PeerNewCmpctBlock>();
        let block_relay_recipient = ctx.address().recipient::<BlockRelayEvent>();
        self.bus
            .send(Subscription {
                recipient: cmpctblock_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        self.bus
            .send(Subscription {
                recipient: block_relay_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
    }
}

impl<C, P, N> Handler<BlockRelayEvent> for BlockRelayer<C, P, N>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    N: NetworkService + Sync + Send + 'static,
{
    type Result = ();
    fn handle(&mut self, event: BlockRelayEvent, _ctx: &mut Self::Context) -> Self::Result {
        let compact_block = self.block_into_compact(event.block);
        let block_detail = Arc::new(BlockDetail::from_compact_block(
            compact_block,
            event.total_difficulty,
        ));
        let sys_new_head_block = system_events::NewHeadBlock(block_detail);
        self.broadcast(sys_new_head_block);
    }
}

impl<C, P, N> Handler<PeerNewCmpctBlock> for BlockRelayer<C, P, N>
where
    C: Consensus + Sync + Send + 'static,
    P: TxPoolSyncService + Sync + Send + 'static,
    N: NetworkService + Sync + Send + 'static,
{
    type Result = ();
    fn handle(
        &mut self,
        cmpct_block_msg: PeerNewCmpctBlock,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let compact_block = cmpct_block_msg.compact_block;
        let peer_id = cmpct_block_msg.peer_id;
        if let Ok(block) = self.fill_compact_block(compact_block) {
            self.bus.do_send(Broadcast {
                msg: PeerNewBlock::new(peer_id, block),
            });
        }
    }
}
