use actix::prelude::*;
use anyhow::Result;
use bus::Bus;
use bus::{Broadcast, BusActor, Subscription};
use crypto::HashValue;
use logger::prelude::*;
use starcoin_block_relayer_api::{NetCmpctBlockMessage, PeerCmpctBlockEvent};
use starcoin_sync_api::PeerNewBlock;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    block::{Block, BlockBody},
    cmpact_block::{CompactBlock, PrefiledTxn, ShortId},
    system_events::NewHeadBlock,
    transaction::{SignedUserTransaction, Transaction},
};
use std::collections::HashMap;
use std::iter::FromIterator;

pub struct BlockRelayer<P>
where
    P: TxPoolSyncService + Sync + Send + 'static,
{
    bus: Addr<BusActor>,
    txpool: P,
}

impl<P> BlockRelayer<P>
where
    P: TxPoolSyncService + Sync + Send + 'static,
{
    pub fn new(bus: Addr<BusActor>, txpool: P) -> Result<Addr<BlockRelayer<P>>> {
        let block_relayer = BlockRelayer { bus, txpool };
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

impl<P> Actor for BlockRelayer<P>
where
    P: TxPoolSyncService + Sync + Send + 'static,
{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let new_head_block_recipient = ctx.address().recipient::<NewHeadBlock>();
        self.bus
            .clone()
            .subscribe(new_head_block_recipient)
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        let cmpctblock_recipient = ctx.address().recipient::<PeerCmpctBlockEvent>();

        self.bus
            .send(Subscription {
                recipient: cmpctblock_recipient,
            })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
    }
}

impl<P> Handler<NewHeadBlock> for BlockRelayer<P>
where
    P: TxPoolSyncService + Sync + Send + 'static,
{
    type Result = ();
    fn handle(&mut self, event: NewHeadBlock, ctx: &mut Self::Context) -> Self::Result {
        debug!("Handle relay new head block event");
        let compact_block = self.block_into_compact(event.0.get_block().clone());
        let total_difficulty = event.0.get_total_difficulty();
        let net_cmpct_block_msg = NetCmpctBlockMessage {
            compact_block,
            total_difficulty,
        };
        self.bus
            .clone()
            .broadcast(net_cmpct_block_msg)
            .into_actor(self)
            .then(|res, act, _ctx| {
                if let Err(e) = res {
                    error!(
                        "Failed to emit new compact block relay message, err: {}",
                        &e
                    );
                }
                async {}.into_actor(act)
            })
            .wait(ctx);
    }
}

impl<P> Handler<PeerCmpctBlockEvent> for BlockRelayer<P>
where
    P: TxPoolSyncService + Sync + Send + 'static,
{
    type Result = ();
    fn handle(
        &mut self,
        cmpct_block_msg: PeerCmpctBlockEvent,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let compact_block = cmpct_block_msg.compact_block;
        let peer_id = cmpct_block_msg.peer_id;
        debug!("Receive peer compact block event from peer id:{}", peer_id);
        if let Ok(block) = self.fill_compact_block(compact_block) {
            self.bus.do_send(Broadcast {
                msg: PeerNewBlock::new(peer_id, block),
            });
        }
    }
}
