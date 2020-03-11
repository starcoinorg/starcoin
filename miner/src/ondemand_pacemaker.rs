// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;

use futures::channel::mpsc;

use super::TransactionStatusEvent;
use bus::{BusActor, Subscription};
use crypto::hash::HashValue;
use futures::stream::StreamExt;
use logger::prelude::*;
use std::{sync::Arc, time::Duration};
use traits::TxPoolAsyncService;
use txpool::TxStatus;
use types::system_events::SystemEvents;

/// On-demand generate block, only generate block when new transaction add to tx-pool.
pub(crate) struct OndemandPacemaker {
    bus: Addr<BusActor>,
    sender: mpsc::Sender<GenerateBlockEvent>,
    transaction_receiver: Option<mpsc::UnboundedReceiver<TransactionStatusEvent>>,
}

impl OndemandPacemaker {
    pub fn new(
        bus: Addr<BusActor>,
        sender: mpsc::Sender<GenerateBlockEvent>,
        transaction_receiver: mpsc::UnboundedReceiver<TransactionStatusEvent>,
    ) -> Self {
        Self {
            bus,
            sender,
            transaction_receiver: Some(transaction_receiver),
        }
    }

    pub fn send_event(&mut self) {
        // TODO handle result.
        self.sender.try_send(GenerateBlockEvent {});
    }
}

impl Actor for OndemandPacemaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<SystemEvents>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        ctx.add_stream(self.transaction_receiver.take().unwrap());
        info!("ondemand pacemaker started.");
    }
}

impl Handler<SystemEvents> for OndemandPacemaker {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SystemEvents::NewUserTransaction(_txn) => self.send_event(),
            _ => {}
        }
    }
}

impl StreamHandler<TransactionStatusEvent> for OndemandPacemaker {
    fn handle(&mut self, tx_item: Arc<Vec<(HashValue, TxStatus)>>, ctx: &mut Self::Context) {
        tx_item.iter().for_each(|(tx, tx_status)| {
            if tx_status.clone() == TxStatus::Added {
                self.send_event();
            }
        });
    }
}
