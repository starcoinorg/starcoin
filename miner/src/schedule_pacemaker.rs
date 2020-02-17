// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GenerateBlockEvent;
use actix::prelude::*;

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
        //TODO handle result.
        self.sender.try_send(GenerateBlockEvent {});
    }
}

impl Actor for SchedulePacemaker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(self.duration, move |act, _ctx| {
            act.send_event();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{future, TryStreamExt};

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
        std::thread::sleep(duration * 2);
        let _result = receiver.try_next().expect("To receive response in time");
    }
}
