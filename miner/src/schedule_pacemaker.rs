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
            Err(e) => trace!("Send GenerateBlockEvent error: {:?}", e),
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
    use futures::channel::oneshot;
    use futures::executor::block_on;
    use std::thread::sleep;

    #[test]
    fn test_schedule_pacemaker() {
        logger::init_for_test();
        let (sender, mut receiver) = mpsc::channel(100);
        let duration = Duration::from_millis(10);
        let (stop_sender, stop_receiver) = oneshot::channel();
        let handle = std::thread::spawn(move || {
            let mut system = System::new("test");
            system.block_on(async move {
                let _addr = SchedulePacemaker::new(duration, sender).start();
                stop_receiver.await.unwrap();
            });
        });
        let mut count = 0;
        loop {
            match receiver.try_next() {
                Err(_) => {
                    debug!("wait event.");
                    sleep(duration * 2)
                }
                Ok(_event) => {
                    debug!("receive event");
                    count = count + 1;
                    if count > 3 {
                        stop_sender.send(()).unwrap();
                        break;
                    }
                }
            }
        }
        handle.join().unwrap();
    }
}
