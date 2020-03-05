// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;

use futures::channel::mpsc;

use bus::{BusActor, Subscription};
use std::time::Duration;
use types::system_events::SystemEvents;
use txpool::{SubscribeTxns, TxStatus};
use std::sync::Arc;
use crypto::hash::HashValue;
use futures::stream::StreamExt;
use traits::TxPoolAsyncService;
use futures::{
    compat::{Future01CompatExt, Stream01CompatExt},
};

/// On-demand generate block, only generate block when new transaction add to tx-pool.
pub(crate) struct OndemandPacemaker<P>
    where P: TxPoolAsyncService + 'static {
    bus: Addr<BusActor>,
    sender: mpsc::Sender<GenerateBlockEvent>,
    txpool: P, //Option<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>>,
    tx: Option<mpsc::UnboundedReceiver<Arc<Vec<(HashValue, TxStatus)>>>>,
}

impl<P> OndemandPacemaker<P>
    where P: TxPoolAsyncService, {
    pub fn new(bus: Addr<BusActor>, sender: mpsc::Sender<GenerateBlockEvent>, txpool: P) -> Self {
        //ctx.add_stream(rx.fuse().compat());

        // let tmp = txpool.clone();
        // let fut = async move {
        //     tmp.subscribe_txns().await.unwrap()
        // };
        //
        // let tx = System::builder().build().block_on(fut);

        // let f = actix::fut::wrap_future(fut);
        // ctx.spawn(Box::new(f));

        // OndemandPacemaker::create(move |ctx: &mut Context<OndemandPacemaker<P>>| {
        //     // let mut rx = txpool.clone().subscribe_txns().await.unwrap();
        //     // ctx.add_stream(rx);
        //
        //     let tmp = txpool.clone();
        //     let fut = async move {
        //         tmp.subscribe_txns().await.unwrap()
        //     };
        //
        //     let tx = System::builder().build().block_on(fut);
        //     ctx.add_stream(tx);
        //
        //
        //     // let f = actix::fut::wrap_future(fut);
        //     // ctx.spawn(Box::new(f));
        //
        //     Self { bus, sender, txpool }
        // })

        Self { bus, sender, txpool, tx: None }
    }

    pub fn send_event(&mut self) {
        //TODO handle result.
        self.sender.try_send(GenerateBlockEvent {});
    }
}

impl<P> Actor for OndemandPacemaker<P>
    where P: TxPoolAsyncService, {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<SystemEvents>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);
        let txpool = self.txpool.clone();


        // let fut = async move {
        //     let mut rx = txpool.subscribe_txns().await.unwrap();
        //     ctx.clone().add_stream(rx);
        //     // loop {
        //     //     println!("receive tx in future.");
        //     //
        //     //     ::futures::select! {
        //     //         event = tx_receiver.select_next_some() => {
        //     //             println!("receive tx in future.");
        //     //         }
        //     //         complete => {
        //     //             break;
        //     //         }
        //     //     }
        //     // }
        // };
        //
        // let f = actix::fut::wrap_future(fut);
        // ctx.spawn(Box::new(f));
        println!("ondemand pacemaker started.");
    }
}

impl<P> Handler<SystemEvents> for OndemandPacemaker<P>
    where P: TxPoolAsyncService, {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SystemEvents::NewUserTransaction(_txn) => self.send_event(),
            _ => {}
        }
    }
}

impl<P> StreamHandler<Arc<Vec<(HashValue, TxStatus)>>> for OndemandPacemaker<P>
    where P: TxPoolAsyncService, {
    fn handle(&mut self, item: Arc<Vec<(HashValue, TxStatus)>>, ctx: &mut Self::Context) {
        unimplemented!()
    }
}
