use crossbeam::channel::{self, Receiver, Sender};
use starcoin_logger::prelude::warn;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_types::system_events::{NewHeadBlock, SystemStarted};

#[derive(Clone, Debug)]
pub struct ProcessNewHeadBlock;

#[derive(Clone)]
pub struct NewHeaderChannel {
    pub new_header_sender: Sender<NewHeadBlock>,
    pub new_header_receiver: Receiver<NewHeadBlock>,
}

impl NewHeaderChannel {
    pub fn new() -> Self {
        let (new_header_sender, new_header_receiver): (
            Sender<NewHeadBlock>,
            Receiver<NewHeadBlock>,
        ) = channel::bounded(2);
        Self {
            new_header_sender,
            new_header_receiver,
        }
    }
}

impl Default for NewHeaderChannel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NewHeaderService {
    new_header_channel: NewHeaderChannel,
}

impl NewHeaderService {
    pub fn new(new_header_channel: NewHeaderChannel) -> Self {
        Self { new_header_channel }
    }
}

impl ActorService for NewHeaderService {
    fn started(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.set_mailbox_capacity(3602); // the merge depth + 2
        ctx.subscribe::<SystemStarted>();
        ctx.subscribe::<NewHeadBlock>();

        Ok(())
    }

    fn stopped(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.unsubscribe::<SystemStarted>();
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl ServiceFactory<Self> for NewHeaderService {
    fn create(ctx: &mut starcoin_service_registry::ServiceContext<Self>) -> anyhow::Result<Self> {
        anyhow::Ok(Self::new(ctx.get_shared::<NewHeaderChannel>()?))
    }
}

impl EventHandler<Self, SystemStarted> for NewHeaderService {
    fn handle_event(&mut self, _: SystemStarted, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(ProcessNewHeadBlock);
    }
}

impl EventHandler<Self, NewHeadBlock> for NewHeaderService {
    fn handle_event(&mut self, msg: NewHeadBlock, _ctx: &mut ServiceContext<Self>) {
        let _consume = self
            .new_header_channel
            .new_header_receiver
            .try_iter()
            .count();
        match self.new_header_channel.new_header_sender.send(msg) {
            Ok(()) => (),
            Err(e) => {
                warn!(
                    "Failed to send new head block: {:?} in BlockBuilderService",
                    e
                );
            }
        }
    }
}
