use std::{sync::Arc, time::Duration};

use anyhow::{format_err, Ok};
use crossbeam::channel::{self, Receiver, Sender};
use starcoin_dag::{blockdag::BlockDAG, types::ghostdata::GhostdagData};
use starcoin_logger::prelude::{error, info, warn};
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{storage, BlockStore, Storage};
use starcoin_types::{
    block::BlockHeader,
    startup_info::StartupInfo,
    system_events::{DeterminedDagBlock, NewDagBlock, NewDagBlockFromPeer, SystemStarted},
};

#[derive(Clone, Debug)]
pub struct ProcessNewHeadBlock;

#[derive(Clone)]
pub struct NewHeaderChannel {
    pub new_header_sender: Sender<Arc<BlockHeader>>,
    pub new_header_receiver: Receiver<Arc<BlockHeader>>,
}

impl NewHeaderChannel {
    pub fn new() -> Self {
        let (new_header_sender, new_header_receiver): (
            Sender<Arc<BlockHeader>>,
            Receiver<Arc<BlockHeader>>,
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
    pub fn new(
        new_header_channel: NewHeaderChannel,
        header: BlockHeader,
        ghostdag_data: GhostdagData,
        dag: BlockDAG,
    ) -> Self {
        Self {
            new_header_channel,
            header,
            ghostdag_data,
            dag,
        }
    }
}

impl ActorService for NewHeaderService {
    fn started(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        let merge_depth = self.dag.block_depth_manager().merge_depth();
        ctx.set_mailbox_capacity(usize::try_from(merge_depth.saturating_add(2))?); // the merge depth + 2
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
        let header_id = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("no startup info when creating NewHeaderService"))?
            .main;
        let header = storage
            .get_block_header_by_hash(header_id)?
            .ok_or_else(|| {
                format_err!(
                    "no main block: {:?} when creating NewHeaderService",
                    header_id
                )
            })?;
        let dag = ctx.get_shared::<BlockDAG>()?;
        let ghostdag_data = dag
            .ghostdata_by_hash(header_id)?
            .ok_or_else(|| {
                format_err!(
                    "no ghostdag data: {:?} when creating NewHeaderService",
                    header_id
                )
            })?
            .as_ref()
            .clone();
        anyhow::Ok(Self::new(
            ctx.get_shared::<NewHeaderChannel>()?,
            header,
            ghostdag_data,
            dag,
        ))
    }
}

impl NewHeaderService {
    fn resolve_header(&mut self, header: &BlockHeader) -> anyhow::Result<bool> {
        info!(
            "resolve_header: new header: {:?}, current header: {:?}",
            header.id(),
            self.header.id()
        );

        if header.id() == self.header.id() {
            return Ok(false);
        }
        let new_ghostdata = self
            .dag
            .ghostdata_by_hash(header.id())?
            .ok_or_else(|| {
                format_err!(
                    "no ghostdag data: {:?} when creating NewHeaderService",
                    header.id()
                )
            })?
            .as_ref()
            .clone();
        let update = match new_ghostdata.blue_work.cmp(&self.ghostdag_data.blue_work) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                match new_ghostdata.blue_score.cmp(&self.ghostdag_data.blue_score) {
                    std::cmp::Ordering::Equal | std::cmp::Ordering::Less => false,
                    std::cmp::Ordering::Greater => true,
                }
            }
            std::cmp::Ordering::Greater => true,
        };

        if update {
            self.header = header.clone();
            self.ghostdag_data = new_ghostdata;
        }

        Ok(update)
    }

    fn determine_header(
        &mut self,
        header: &BlockHeader,
        ctx: &mut ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        info!("jacktest: new dag block, determine_header: new header: {:?}, number: {:?}, current header: {:?}, number: {:?}", header.id(), header.number(), self.header.id(), self.header.number());
        if self.resolve_header(header)? {
            info!(
                "resolve header returns true, header: {:?} will be sent to BlockBuilderService",
                header.id()
            );
            let _consume = self
                .new_header_channel
                .new_header_receiver
                .try_iter()
                .count();
            match self
                .new_header_channel
                .new_header_sender
                .send(Arc::new(self.header.clone()))
            {
                anyhow::Result::Ok(()) => (),
                Err(e) => {
                    warn!("Failed to send new head block: {:?} in NewHeaderService", e);
                }
            }
        } else {
            info!("resolve header returns false");
        }

        ctx.broadcast(DeterminedDagBlock);

        Ok(())
    }
}

impl EventHandler<Self, SystemStarted> for NewHeaderService {
    fn handle_event(&mut self, _: SystemStarted, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(ProcessNewHeadBlock);
    }
}

impl EventHandler<Self, NewDagBlockFromPeer> for NewHeaderService {
    fn handle_event(&mut self, msg: NewDagBlockFromPeer, ctx: &mut ServiceContext<Self>) {
        info!(
            "handle_event: NewDagBlockFromPeer, msg: {:?}",
            msg.executed_block.id()
        );
        match self.determine_header(msg.executed_block.as_ref(), ctx) {
            anyhow::Result::Ok(()) => (),
            Err(e) => error!(
                "Failed to determine header: {:?} when processing NewDagBlockFromPeer",
                e
            ),
        }
    }
}

impl EventHandler<Self, NewDagBlock> for NewHeaderService {
    fn handle_event(&mut self, msg: NewDagBlock, ctx: &mut ServiceContext<Self>) {
        info!(
            "handle_event: NewDagBlock, msg: {:?}",
            msg.executed_block.header().id()
        );
        match self.determine_header(msg.executed_block.header(), ctx) {
            anyhow::Result::Ok(()) => (),
            Err(e) => error!(
                "Failed to determine header: {:?} when processing NewDagBlock",
                e
            ),
        }
    }
}
