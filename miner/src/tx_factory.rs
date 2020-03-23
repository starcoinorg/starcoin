use actix::prelude::*;
use anyhow::Result;
use executor::executor::mock_create_account_txn;
use std::convert::TryInto;
use traits::TxPoolAsyncService;
use types::account_address::AccountAddress;

#[derive(Default, Debug, Message)]
#[rtype(result = "Result<()>")]
pub(crate) struct GenTxEvent {}

/// generate transaction just for test.
pub(crate) struct TxFactoryActor<P>
where
    P: TxPoolAsyncService + 'static,
{
    txpool: P,
}

impl<P> TxFactoryActor<P>
where
    P: TxPoolAsyncService,
{
    pub fn _launch(txpool: P) -> Result<Addr<Self>> {
        let actor = TxFactoryActor { txpool };
        Ok(actor.start())
    }
}

impl<P> Actor for TxFactoryActor<P>
where
    P: TxPoolAsyncService,
{
    type Context = Context<Self>;
}

impl<P> Handler<GenTxEvent> for TxFactoryActor<P>
where
    P: TxPoolAsyncService,
{
    type Result = Result<()>;

    fn handle(&mut self, _event: GenTxEvent, ctx: &mut Self::Context) -> Self::Result {
        let txpool = self.txpool.clone();
        let f = async {
            let tx = mock_create_account_txn();
            txpool.add(tx.try_into().unwrap()).await.unwrap();
        }
        .into_actor(self);
        ctx.spawn(f);
        Ok(())
    }
}
