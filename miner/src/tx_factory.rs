use crate::mock_txn_generator::MockTxnGenerator;
use actix::prelude::*;
use anyhow::Result;
use bus::Bus;
use bus::BusActor;
use logger::prelude::*;
use statedb::ChainStateDB;
use std::convert::TryInto;
use std::sync::Arc;
use storage::BlockChainStore;
use traits::TxPoolAsyncService;
use types::account_config;
use types::block::BlockHeader;
use types::system_events::SystemEvents;
use types::transaction::{SignedUserTransaction, Transaction};

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct GenTxEvent;

/// generate transaction just for test.
pub(crate) struct TxFactoryActor<P, TStorage> {
    txpool: P,
    storage: Arc<TStorage>,
    mock_txn_generator: MockTxnGenerator,
    bus: Addr<BusActor>,
    best_block_header: Option<BlockHeader>,
}

impl<P, TStorage> TxFactoryActor<P, TStorage>
where
    P: TxPoolAsyncService + 'static,
    TStorage: BlockChainStore + Sync + Send + 'static,
{
    pub fn launch(txpool: P, storage: Arc<TStorage>, bus: Addr<BusActor>) -> Result<Addr<Self>> {
        let actor = TxFactoryActor {
            txpool,
            storage,
            bus,
            best_block_header: None,
            mock_txn_generator: MockTxnGenerator::new(account_config::association_address()),
        };
        Ok(actor.start())
    }

    fn gen_mock_txn(&self) -> Result<Option<Transaction>> {
        let block_header = self.best_block_header.as_ref();
        if block_header.is_none() {
            return Ok(None);
        }
        let block_header = block_header.unwrap();
        let lastest_state_root = block_header.state_root();
        let state_db = ChainStateDB::new(self.storage.clone(), Some(lastest_state_root));

        self.mock_txn_generator
            .generate_mock_txn(&state_db)
            .map(|t| Some(t))
    }
}

impl<P, TStorage> Actor for TxFactoryActor<P, TStorage>
where
    P: TxPoolAsyncService + 'static,
    TStorage: BlockChainStore + Sync + Send + 'static,
{
    type Context = Context<Self>;
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

        info!("txn factory started");
    }
}

impl<P, TStorage> actix::Handler<SystemEvents> for TxFactoryActor<P, TStorage>
where
    P: TxPoolAsyncService + 'static,
    TStorage: BlockChainStore + Sync + Send + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SystemEvents::NewHeadBlock(block) => {
                info!(
                    "tx_factory: aware best block changed, block number: {}, difficulty: {}",
                    block.header().number(),
                    block.header().difficult()
                );
                self.best_block_header = Some(block.into_inner().0);
            }
            _ => {}
        }
    }
}

impl<P, TStorage> Handler<GenTxEvent> for TxFactoryActor<P, TStorage>
where
    P: TxPoolAsyncService + 'static,
    TStorage: BlockChainStore + Sync + Send + 'static,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenTxEvent, ctx: &mut Self::Context) -> Self::Result {
        let txn = self.gen_mock_txn();
        let txn = match txn {
            Err(err) => {
                error!("fail to gen mock txn, error: {}", &err);
                // return Err(err);
                return Ok(());
            }
            Ok(Some(txn)) => {
                let txn: SignedUserTransaction = txn.try_into().unwrap();
                txn
            }
            Ok(None) => {
                debug!("skip gen txn, not aware best block header");
                return Ok(());
            }
        };

        // info!("gen test txn: {:?}", tx);
        let txpool = self.txpool.clone();
        let f = async move {
            let txn_sender = txn.sender();
            let seq_number = txn.sequence_number();
            info!("gen_tx_for_test call txpool.");
            match txpool.add_txns(vec![txn]).await {
                Ok(mut t) => {
                    let result = t.pop().unwrap();
                    if let Err(e) = result {
                        error!(
                            "fail to add txn(sender:{},seq:{}) to txpool, err: {}",
                            txn_sender, seq_number, &e
                        )
                    } else {
                        debug!(
                            "succ to add txn(sender:{},seq:{}) to txpool",
                            txn_sender, seq_number
                        );
                    }
                }
                Err(e) => {
                    error!("fail to call add_txns on txpool, err: {}", &e);
                }
            };
        };
        f.into_actor(self).wait(ctx);
        Ok(())
    }
}
