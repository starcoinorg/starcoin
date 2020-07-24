use actix::prelude::*;
use anyhow::Result;
use bus::Bus;
use bus::{Broadcast, BusActor, Subscription};
use crypto::HashValue;
use logger::prelude::*;
use network_rpc::{gen_client::NetworkRpcClient, GetTxns};
use starcoin_block_relayer_api::{NetCmpctBlockMessage, PeerCmpctBlockEvent};
use starcoin_network::network::NetworkAsyncService;
use starcoin_sync::helper::get_txns;
use starcoin_sync_api::PeerNewBlock;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    block::{Block, BlockBody},
    cmpact_block::{CompactBlock, PrefiledTxn, ShortId},
    peer_info::PeerId,
    system_events::NewHeadBlock,
    transaction::{SignedUserTransaction, Transaction},
};
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

pub struct BlockRelayer<P>
where
    P: TxPoolSyncService + Sync + Send + 'static,
{
    bus: Addr<BusActor>,
    txpool: P,
    rpc_client: NetworkRpcClient<NetworkAsyncService>,
}

impl<P> BlockRelayer<P>
where
    P: TxPoolSyncService + Sync + Send + 'static,
{
    pub fn new(
        bus: Addr<BusActor>,
        txpool: P,
        network: NetworkAsyncService,
    ) -> Result<Addr<BlockRelayer<P>>> {
        let block_relayer = BlockRelayer {
            bus,
            txpool,
            rpc_client: NetworkRpcClient::new(network),
        };
        Ok(block_relayer.start())
    }

    async fn fill_compact_block(
        txpool: P,
        rpc_client: NetworkRpcClient<NetworkAsyncService>,
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
                        txns[index] = Some(txn.clone());
                    }
                }
            }
            txns.iter()
                .filter(|&txn| txn.is_some())
                .map(|txn| txn.clone().unwrap())
                .collect()
        };
        let body = BlockBody::new(txns, None);
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
        let bus = self.bus.clone();
        let rpc_client = self.rpc_client.clone();
        let txpool = self.txpool.clone();
        let fut = async move {
            let compact_block = cmpct_block_msg.compact_block;
            let peer_id = cmpct_block_msg.peer_id;
            debug!("Receive peer compact block event from peer id:{}", peer_id);
            if let Ok(block) =
                BlockRelayer::fill_compact_block(txpool, rpc_client, compact_block, peer_id.clone())
                    .await
            {
                bus.do_send(Broadcast {
                    msg: PeerNewBlock::new(peer_id, block),
                });
            }
        };
        Arbiter::spawn(fut);
    }
}
