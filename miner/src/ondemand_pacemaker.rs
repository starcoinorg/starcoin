// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;

use futures::channel::mpsc;

use super::TransactionStatusEvent;
use bus::BusActor;
use crypto::hash::HashValue;
use logger::prelude::*;
use std::sync::Arc;
use txpool::TxStatus;

/// On-demand generate block, only generate block when new transaction add to tx-pool.
pub(crate) struct OndemandPacemaker {
    _bus: Addr<BusActor>,
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
            _bus: bus,
            sender,
            transaction_receiver: Some(transaction_receiver),
        }
    }

    pub fn send_event(&mut self) {
        if let Err(e) = self.sender.try_send(GenerateBlockEvent::new(false)) {
            trace!("err : {:?}", e);
        }
    }
}

impl Actor for OndemandPacemaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.add_stream(self.transaction_receiver.take().unwrap());
        info!("OndemandPacemaker started.");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("OndemandPacemaker stopped");
    }
}

impl StreamHandler<TransactionStatusEvent> for OndemandPacemaker {
    fn handle(&mut self, _tx_item: Arc<Vec<(HashValue, TxStatus)>>, _ctx: &mut Self::Context) {
        self.send_event();
    }
}
