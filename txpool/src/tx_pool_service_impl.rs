// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters::{TXPOOL_STATUS_GAUGE_VEC, TXPOOL_TXNS_GAUGE};
use crate::pool::VerifiedTransaction;
use crate::{
    pool,
    pool::{
        Gas, PendingOrdering, PendingSettings, PoolTransaction, PrioritizationStrategy, TxStatus,
        UnverifiedUserTransaction,
    },
    pool_client::{NonceCache, PoolClient},
};
use actix::prelude::*;
use anyhow::Result;
use common_crypto::hash::{HashValue, PlainCryptoHash};
use futures_channel::mpsc;
use starcoin_bus::{Bus, BusActor};
use starcoin_config::TxPoolConfig;
use std::sync::Arc;
use storage::Store;
use tx_relay::{PeerTransactions, PropagateNewTransactions};
use types::{
    account_address::AccountAddress, block::BlockHeader, system_events::SystemEvents, transaction,
    transaction::SignedUserTransaction,
};

type TxnQueue = pool::TransactionQueue;
#[derive(Clone)]
pub(crate) struct TxPoolActor {
    queue: Arc<TxnQueue>,
    chain_header: BlockHeader,
    storage: Arc<dyn Store>,
    sequence_number_cache: NonceCache,
    bus: actix::Addr<BusActor>,
}
impl std::fmt::Debug for TxPoolActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "pool: {:?}, chain header: {:?} bus: {:?}",
            &self.queue,
            &self.chain_header,
            self.bus.connected()
        )
    }
}

impl TxPoolActor {
    pub fn new(
        pool_config: TxPoolConfig,
        storage: Arc<dyn Store>,
        chain_header: BlockHeader,
        bus: actix::Addr<BusActor>,
    ) -> Self {
        let verifier_options = pool::VerifierOptions {
            minimal_gas_price: pool_config.minimal_gas_price,
            block_gas_limit: Gas::max_value(),
            tx_gas_limit: pool_config.tx_gas_limit,
            no_early_reject: false,
        };
        let queue = TxnQueue::new(
            tx_pool::Options {
                max_count: pool_config.max_count as usize,
                max_mem_usage: pool_config.max_mem_usage as usize,
                max_per_sender: pool_config.max_per_sender as usize,
            },
            verifier_options,
            PrioritizationStrategy::GasPriceOnly,
        );
        let queue = Arc::new(queue);
        Self {
            queue,
            storage,
            chain_header,
            bus,
            sequence_number_cache: NonceCache::new(128),
        }
    }
    fn get_pending(&self, max_len: u64) -> Vec<Arc<VerifiedTransaction>> {
        let pending_settings = PendingSettings {
            block_number: u64::max_value(),
            current_timestamp: u64::max_value(),
            nonce_cap: None,
            max_len: max_len as usize,
            ordering: PendingOrdering::Priority,
        };
        let client = PoolClient::new(
            self.chain_header.clone(),
            self.storage.clone(),
            self.sequence_number_cache.clone(),
        );
        self.queue.pending(client, pending_settings)
    }
}

impl actix::Actor for TxPoolActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // subscribe system block event
        let myself = ctx.address().recipient::<SystemEvents>();
        self.bus
            .clone()
            .subscribe(myself)
            .into_actor(self)
            .then(|res, act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe system events, err: {:?}", e);
                    ctx.terminate();
                }
                async {}.into_actor(act)
            })
            .wait(ctx);

        // subscribe txn relay peer txns
        let myself = ctx.address().recipient::<PeerTransactions>();
        self.bus
            .clone()
            .subscribe(myself)
            .into_actor(self)
            .then(|res, act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe txn relay message, err: {:?}", e);
                    ctx.terminate();
                }
                async {}.into_actor(act)
            })
            .wait(ctx);

        let receiver = {
            let (tx, rx) = mpsc::unbounded();
            self.queue.add_full_listener(tx);
            rx
        };
        ctx.add_stream(receiver);

        info!("txn pool started");
    }
}
type TxnStatusEvent = Arc<Vec<(HashValue, TxStatus)>>;
/// Listen to txn status, and propagate to remote peers if necessary.
impl StreamHandler<TxnStatusEvent> for TxPoolActor {
    fn handle(&mut self, item: TxnStatusEvent, ctx: &mut Context<Self>) {
        {
            let status = self.queue.status().status;
            let mem_usage = status.mem_usage;
            let senders = status.senders;
            let txn_count = status.transaction_count;
            TXPOOL_STATUS_GAUGE_VEC
                .with_label_values(&["mem_usage"])
                .set(mem_usage as i64);
            TXPOOL_STATUS_GAUGE_VEC
                .with_label_values(&["senders"])
                .set(senders as i64);
            TXPOOL_STATUS_GAUGE_VEC
                .with_label_values(&["count"])
                .set(txn_count as i64);
        }
        // TODO: need peer info to do more accurate sending.
        let mut txns = vec![];
        for (h, s) in item.iter() {
            match *s {
                TxStatus::Added => {
                    TXPOOL_TXNS_GAUGE.inc();
                }
                TxStatus::Rejected => {}
                _ => {
                    TXPOOL_TXNS_GAUGE.dec();
                }
            }

            if *s != TxStatus::Added {
                continue;
            }

            if let Some(txn) = self.queue.find(h) {
                txns.push(txn.signed().clone());
            }
        }
        if txns.is_empty() {
            return;
        }
        self.bus
            .clone()
            .broadcast(PropagateNewTransactions::from(txns))
            .into_actor(self)
            .then(|res, act, _ctx| {
                if let Err(e) = res {
                    error!("fail to emit propagate new txn event, err: {}", &e);
                }
                async {}.into_actor(act)
            })
            .wait(ctx);
    }
}

impl actix::Handler<SystemEvents> for TxPoolActor {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) -> Self::Result {
        if let SystemEvents::NewHeadBlock(block) = msg {
            self.chain_header = block.get_block().header().clone();
            self.sequence_number_cache.clear();

            // NOTICE: as the new head block event is sepeated with chain_new_block event,
            // we need to remove invalid txn here.
            // In fact, it would be better if caller can make it into one.
            // In this situation, we don't need to reimport invalid txn on chain_new_block.
            let client = PoolClient::new(
                self.chain_header.clone(),
                self.storage.clone(),
                self.sequence_number_cache.clone(),
            );
            self.queue.cull(client)
        }
    }
}

impl actix::Handler<PeerTransactions> for TxPoolActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: PeerTransactions,
        ctx: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        // JUST need to keep at most once delivery.
        let txns = msg.peer_transactions();
        ctx.notify(ImportTxns { txns })
    }
}

pub(crate) struct ImportTxns {
    pub(crate) txns: Vec<transaction::SignedUserTransaction>,
}

impl actix::Message for ImportTxns {
    type Result = Vec<Result<(), transaction::TransactionError>>;
}
impl actix::Handler<ImportTxns> for TxPoolActor {
    type Result = actix::MessageResult<ImportTxns>;

    fn handle(&mut self, msg: ImportTxns, _ctx: &mut Self::Context) -> Self::Result {
        let ImportTxns { txns } = msg;

        let txns = txns
            .into_iter()
            .map(|t| PoolTransaction::Unverified(UnverifiedUserTransaction::from(t)));
        let client = PoolClient::new(
            self.chain_header.clone(),
            self.storage.clone(),
            self.sequence_number_cache.clone(),
        );
        let import_result = { self.queue.import(client, txns) };
        actix::MessageResult(import_result)
    }
}

pub(crate) struct RemoveTxn {
    pub(crate) txn_hash: HashValue,
    pub(crate) is_invalid: bool,
}
impl actix::Message for RemoveTxn {
    type Result = Option<Arc<pool::VerifiedTransaction>>;
}
impl actix::Handler<RemoveTxn> for TxPoolActor {
    type Result = actix::MessageResult<RemoveTxn>;

    fn handle(&mut self, msg: RemoveTxn, _ctx: &mut Self::Context) -> Self::Result {
        let RemoveTxn {
            txn_hash,
            is_invalid,
        } = msg;
        let mut removed = self.queue.remove(vec![&txn_hash], is_invalid);
        let removed = removed
            .pop()
            .expect("remove should return one result per hash");
        actix::MessageResult(removed)
    }
}

pub(crate) struct GetPendingTxns {
    pub(crate) max_len: u64,
}

impl actix::Message for GetPendingTxns {
    type Result = Vec<Arc<pool::VerifiedTransaction>>;
}

impl actix::Handler<GetPendingTxns> for TxPoolActor {
    type Result = actix::MessageResult<GetPendingTxns>;

    fn handle(&mut self, msg: GetPendingTxns, _ctx: &mut Self::Context) -> Self::Result {
        let GetPendingTxns { max_len } = msg;
        let result = self.get_pending(max_len);
        actix::MessageResult(result)
    }
}

pub(crate) struct NextSequenceNumber {
    pub(crate) address: AccountAddress,
}

impl actix::Message for NextSequenceNumber {
    type Result = Option<u64>;
}

impl actix::Handler<NextSequenceNumber> for TxPoolActor {
    type Result = actix::MessageResult<NextSequenceNumber>;

    fn handle(&mut self, msg: NextSequenceNumber, _ctx: &mut Self::Context) -> Self::Result {
        let NextSequenceNumber { address } = msg;
        let client = PoolClient::new(
            self.chain_header.clone(),
            self.storage.clone(),
            self.sequence_number_cache.clone(),
        );
        let result = self.queue.next_sequence_number(client, &address);

        actix::MessageResult(result)
    }
}

pub(crate) struct SubscribeTxns;
impl actix::Message for SubscribeTxns {
    type Result = mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>;
}

impl actix::Handler<SubscribeTxns> for TxPoolActor {
    type Result = actix::MessageResult<SubscribeTxns>;

    fn handle(&mut self, _: SubscribeTxns, _ctx: &mut Self::Context) -> Self::Result {
        let result = {
            let (tx, rx) = mpsc::unbounded();
            self.queue.add_full_listener(tx);
            rx
        };
        actix::MessageResult(result)
    }
}

pub(crate) struct ChainNewBlock {
    pub(crate) enacted: Vec<SignedUserTransaction>,
    pub(crate) retracted: Vec<SignedUserTransaction>,
}
impl actix::Message for ChainNewBlock {
    type Result = Result<()>;
}
impl actix::Handler<ChainNewBlock> for TxPoolActor {
    type Result = <ChainNewBlock as actix::Message>::Result;

    fn handle(&mut self, msg: ChainNewBlock, _ctx: &mut Self::Context) -> Self::Result {
        let ChainNewBlock { enacted, retracted } = msg;

        info!(
            "receive chain_new_block msg, enacted: {:?}, retracted: {:?}",
            enacted
                .iter()
                .map(|t| (t.sender(), t.sequence_number()))
                .collect::<Vec<_>>(),
            retracted
                .iter()
                .map(|t| (t.sender(), t.sequence_number()))
                .collect::<Vec<_>>(),
        );

        let hashes: Vec<_> = enacted.iter().map(|t| t.crypto_hash()).collect();
        let _ = self.queue.remove(hashes.iter(), false);

        let client = PoolClient::new(
            self.chain_header.clone(),
            self.storage.clone(),
            self.sequence_number_cache.clone(),
        );

        let txns = retracted
            .into_iter()
            .map(|t| PoolTransaction::Retracted(UnverifiedUserTransaction::from(t)));
        let _ = self.queue.import(client, txns);
        // ignore import result
        // for r in import_result {
        //     r?;
        // }
        // self.queue.cull(client);
        Ok(())
    }
}
#[cfg(test)]
mod test {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_static<T: 'static>() {}
    #[test]
    fn test_sync_and_send() {
        assert_send::<super::TxPoolActor>();
        assert_sync::<super::TxPoolActor>();
        assert_static::<super::TxPoolActor>();
    }
}
