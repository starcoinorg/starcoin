// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;
use logger::prelude::*;

use futures::channel::mpsc;

use std::time::Duration;

/// Schedule generate block.
pub(crate) struct SchedulePacemaker {
    duration: Duration,
    sender: mpsc::Sender<GenerateBlockEvent>,
}

impl SchedulePacemaker {
    pub fn new(duration: Duration, sender: mpsc::Sender<GenerateBlockEvent>) -> Self {
        Self { duration, sender }
    }

    pub fn send_event(&mut self) {
        match self.sender.try_send(GenerateBlockEvent {}) {
            Ok(()) => {}
            Err(e) => warn!("Send GenerateBlockEvent error: {:?}", e),
        };
    }
}

impl Actor for SchedulePacemaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let duration = self.duration.clone();
        ctx.run_later(duration, move |_act, ctx| {
            ctx.run_interval(duration, move |act, _ctx| {
                info!("send GenerateBlockEvent.");
                act.send_event();
            });
        });
        info!("schedule pacemaker started.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_pacemaker() {
        let (sender, mut receiver) = mpsc::channel(100);
        let duration = Duration::from_millis(10);
        std::thread::spawn(move || {
            System::run(move || {
                let _addr = SchedulePacemaker::new(duration, sender).start();
            })
            .unwrap();
        });
        std::thread::sleep(duration * 3);
        let _result = receiver.try_next().expect("To receive response in time");
    }
}
