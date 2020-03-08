use std::time::Duration;

use actix::prelude::*;
use anyhow::Result;
use executor::mock_executor::mock_mint_txn;
use futures::channel::mpsc;
use state_tree::StateNodeStore;
use statedb::ChainStateDB;
use std::convert::TryInto;
use std::sync::Arc;
use traits::TxPoolAsyncService;
use types::account_address::AccountAddress;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct GenTxEvent {}

/// generate transaction just for test.
pub(crate) struct TxFactoryActor<P, S>
where
    P: TxPoolAsyncService + 'static,
    S: StateNodeStore + 'static,
{
    txpool: P,
    state_node_store: Arc<S>,
}

impl<P, S> TxFactoryActor<P, S>
where
    P: TxPoolAsyncService,
    S: StateNodeStore,
{
    pub fn launch(txpool: P, state_node_store: Arc<S>) -> Result<Addr<Self>> {
        let actor = TxFactoryActor {
            txpool,
            state_node_store,
        };
        Ok(actor.start())
    }
}

impl<P, S> Actor for TxFactoryActor<P, S>
where
    P: TxPoolAsyncService,
    S: StateNodeStore,
{
    type Context = Context<Self>;
}

impl<P, S> Handler<GenTxEvent> for TxFactoryActor<P, S>
where
    P: TxPoolAsyncService,
    S: StateNodeStore,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenTxEvent, ctx: &mut Self::Context) -> Self::Result {
        let state_node_store = Arc::clone(&self.state_node_store);
        let txpool = self.txpool.clone();
        let f = async {
            let chain_state = ChainStateDB::new(state_node_store, None);
            let tx = mock_mint_txn(AccountAddress::random(), 100);
            txpool.add(tx.try_into().unwrap()).await.unwrap();
        }
        .into_actor(self);
        ctx.spawn(f);
        Ok(())
    }
}
