use std::sync::Arc;

use anyhow::{format_err, Ok};
use crossbeam::channel::{self, Receiver, Sender};
use starcoin_crypto::ed25519::ed25519_dalek::ed25519::signature::digest::block_buffer::Block;
use starcoin_dag::{blockdag::BlockDAG, ghostdag, types::ghostdata::GhostdagData};
use starcoin_logger::prelude::{error, warn};
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{BlockStore, Storage};
use starcoin_sync_api::StartSyncTxnEvent;
use starcoin_types::{account_config::dao, block::BlockHeader, startup_info::StartupInfo, system_events::{NewDagBlock, NewDagBlockFromPeer, NewHeadBlock, SystemStarted}};

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
    header: BlockHeader,
    ghostdag_data: GhostdagData,
    dag: BlockDAG,
}

impl NewHeaderService {
    pub fn new(new_header_channel: NewHeaderChannel, header: BlockHeader, ghostdag_data: GhostdagData, dag: BlockDAG) -> Self {
        Self { new_header_channel, header, ghostdag_data, dag }
    }
}

impl ActorService for NewHeaderService {
    fn started(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.set_mailbox_capacity(3602); // the merge depth + 2
        ctx.subscribe::<SystemStarted>();
        ctx.subscribe::<NewDagBlock>();
        ctx.subscribe::<NewDagBlockFromPeer>();

        Ok(())
    }

    fn stopped(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.unsubscribe::<SystemStarted>();
        ctx.unsubscribe::<NewDagBlock>();
        ctx.unsubscribe::<NewDagBlockFromPeer>();
        Ok(())
    }
}

impl ServiceFactory<Self> for NewHeaderService {
    fn create(ctx: &mut starcoin_service_registry::ServiceContext<Self>) -> anyhow::Result<Self> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let header_id = storage.get_startup_info()?.ok_or_else(|| format_err!("no startup info when creating NewHeaderService"))?.main;
        let header = storage.get_block_header_by_hash(header_id)?.ok_or_else(|| format_err!("no main block: {:?} when creating NewHeaderService", header_id))?;
        let dag = ctx.get_shared::<BlockDAG>()?;
        let ghostdag_data = dag.ghostdata_by_hash(header_id)?.ok_or_else(|| format_err!("no ghostdag data: {:?} when creating NewHeaderService", header_id))?.as_ref().clone();
        anyhow::Ok(Self::new(ctx.get_shared::<NewHeaderChannel>()?, header, ghostdag_data, dag))
    }
}

impl NewHeaderService {
    fn resolve_header(&mut self, header: &BlockHeader) -> Result<bool> {
        if header.id() == self.header.id() {
            return Ok(false);
        }
        let new_ghostdata = self.dag.ghostdata_by_hash(header.id())?.ok_or_else(|| format_err!("no ghostdag data: {:?} when creating NewHeaderService", header.id()))?.as_ref().clone();
        let update = match new_ghostdata.blue_work.cmp(&self.ghostdag_data.blue_work) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                match new_ghostdata.blue_score.cmp(&self.ghostdag_data.blue_score) {
                    std::cmp::Ordering::Less => false,
                    std::cmp::Ordering::Equal => {
                        match header.id().cmp(&self.header.id()) {
                            std::cmp::Ordering::Less => false,
                            std::cmp::Ordering::Equal => panic!("same block, this condition should not happen"),
                            std::cmp::Ordering::Greater => true,
                        } 
                    }
                    std::cmp::Ordering::Greater => true,
                }
            }
            std::cmp::Ordering::Greater => true,
        };

        if update {
            self.header = header.clone();
            self.ghostdag_data = new_ghostdata;
        }

        Ok(true)
    }

    fn determine_header(&mut self, header: &BlockHeader) -> Result<()> {
        if self.resolve_header(header)? {
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

        Ok(())
    }
}

impl EventHandler<Self, SystemStarted> for NewHeaderService {
    fn handle_event(&mut self, _: SystemStarted, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(ProcessNewHeadBlock);
    }
}

impl EventHandler<Self, NewDagBlockFromPeer> for NewHeaderService {
    fn handle_event(&mut self, msg: NewDagBlockFromPeer, _ctx: &mut ServiceContext<Self>) {
        match self.determine_header(msg.executed_block.as_ref()) {
            Ok(()) => (),
            Err(e) => error!("Failed to determine header: {:?} when processing NewDagBlockFromPeer", e),
        }
    }
}

impl EventHandler<Self, NewDagBlock> for NewHeaderService {
    fn handle_event(&mut self, msg: NewHeadBlock, _ctx: &mut ServiceContext<Self>) {
        match self.determine_header(msg.executed_block.header()) {
            Ok(()) => (),
            Err(e) => error!("Failed to determine header: {:?} when processing NewDagBlock", e),
        }
    }
}
