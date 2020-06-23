use starcoin_sync_api::SyncNotify::NewPeerMsg;
use starcoin_sync_api::{PeerNewCmpctBlock, SyncNotify};
use actix::{prelude::*, Actor, Addr, Context, Handler};
use bus::{BusActor, Subscription};
use anyhow::Result;
use crate::download::DownloadActor;
use types::{block::{Block, BlockBody}, cmpact_block::{CompactBlock, ShortId}, transaction::{Transaction, SignedUserTransaction}};
use starcoin_txpool_api::TxPoolSyncService;
use traits::Consensus;
use std::sync::Arc;
use config::NodeConfig;
use std::collections::HashMap;
use std::iter::FromIterator;
use itertools::Itertools;

pub struct BlockRelayer<C, P>
    where
        C: Consensus + Sync + Send + 'static,
        P: TxPoolSyncService + Sync + Send + 'static,
{
    config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    download_address: Addr<DownloadActor<C>>,
    txpool: P,

}

impl<C, P> BlockRelayer<C, P>
    where
        C: Consensus + Sync + Send + 'static,
        P: TxPoolSyncService + Sync + Send + 'static,
{
    pub fn new(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        download_address: Addr<DownloadActor<C>>,
        txpool: P,
    ) -> Result<Addr<BlockRelayer<C, P>>> {
        let block_relayer = BlockRelayer {
            config,
            bus,
            download_address,
            txpool,
        };
        Ok(block_relayer.start())
    }

    fn fill_compact_block(&self, compact_block: CompactBlock) -> Result<Block> {
        let txns_pool_vec = self.txpool.get_pending_txns(None);
        let txns_pool_map: HashMap<ShortId, &SignedUserTransaction> = {
            let pool_id_txn_iter = txns_pool_vec.iter()
                .map(|txn| (Transaction::UserTransaction(txn.clone()).id(), txn))
                .map(|(id, txn)| (ShortId(id), txn));
            HashMap::from_iter(pool_id_txn_iter)
        };
        let txns = {
            let mut txns: Vec<SignedUserTransaction> = Vec::with_capacity(compact_block.short_ids.len());
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
}

impl<C, P> Actor for BlockRelayer<C, P>
    where C: Consensus + Sync + Send + 'static,
          P: TxPoolSyncService + Sync + Send + 'static,
{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<PeerNewCmpctBlock>();
        self.bus.send(Subscription {
            recipient,
        }).into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
    }
}


impl<C, P> Handler<PeerNewCmpctBlock> for BlockRelayer<C, P>
    where C: Consensus + Sync + Send + 'static,
          P: TxPoolSyncService + Sync + Send + 'static,
{
    type Result = ();
    fn handle(&mut self, cmpct_block_msg: PeerNewCmpctBlock, ctx: &mut Self::Context) -> Self::Result {
        let compact_block = cmpct_block_msg.compact_block;
        if let Ok(block) = self.fill_compact_block(compact_block) {
            let new_block = SyncNotify::NewHeadBlock(cmpct_block_msg.peer_id, Box::new(block));
            self.download_address
                .send(new_block)
                .into_actor(self)
                .then(|_result, act, _ctx| async {}.into_actor(act))
                .wait(ctx);
        }
    }
}