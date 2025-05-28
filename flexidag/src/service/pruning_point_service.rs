use std::sync::Arc;

use anyhow::format_err;
use crossbeam::channel::{self, Receiver, Sender};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::{error, info, warn};
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_state_api::AccountStateReader;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{BlockStore, IntoSuper, Storage};
use starcoin_types::{block::BlockHeader, system_events::SystemStarted};
use starcoin_vm_types::on_chain_config::FlexiDagConfigV2;

use crate::blockdag::BlockDAG;

#[derive(Clone)]
pub struct PruningPointMessage {
    pub block_header: BlockHeader,
}

#[derive(Clone, Debug)]
pub struct PruningPointInfoGeneration;

#[derive(Clone)]
pub struct PruningPointInfoChannel {
    pub pruning_sender: Sender<PruningPointMessage>,
    pub pruning_receiver: Receiver<PruningPointMessage>,
}

impl PruningPointInfoChannel {
    pub fn new() -> Self {
        let (pruning_sender, pruning_receiver): (
            Sender<PruningPointMessage>,
            Receiver<PruningPointMessage>,
        ) = channel::bounded(2);
        Self {
            pruning_sender,
            pruning_receiver,
        }
    }
}

impl Default for PruningPointInfoChannel {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PruningPointService {
    dag: BlockDAG,
    pruning_channel: PruningPointInfoChannel,
    genesis_id: HashValue,
    storage: Arc<Storage>,
}

impl PruningPointService {
    pub fn new(
        dag: BlockDAG,
        pruning_channel: PruningPointInfoChannel,
        genesis_id: HashValue,
        storage: Arc<Storage>,
    ) -> Self {
        Self {
            dag,
            pruning_channel,
            genesis_id,
            storage,
        }
    }
}

impl ActorService for PruningPointService {
    fn started(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.set_mailbox_capacity(1024);
        ctx.subscribe::<SystemStarted>();
        ctx.subscribe::<PruningPointInfoGeneration>();

        Ok(())
    }

    fn stopped(
        &mut self,
        ctx: &mut starcoin_service_registry::ServiceContext<Self>,
    ) -> anyhow::Result<()> {
        ctx.unsubscribe::<SystemStarted>();
        ctx.unsubscribe::<PruningPointInfoGeneration>();
        Ok(())
    }
}

impl ServiceFactory<Self> for PruningPointService {
    fn create(ctx: &mut starcoin_service_registry::ServiceContext<Self>) -> anyhow::Result<Self> {
        let dag = ctx.get_shared::<BlockDAG>()?;
        let pruning_channel = PruningPointInfoChannel::new();
        ctx.put_shared(pruning_channel.clone())?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let genesis_id = storage
            .get_genesis()?
            .ok_or_else(|| format_err!("genesis not found"))?;
        anyhow::Ok(Self::new(dag, pruning_channel, genesis_id, storage))
    }
}

impl EventHandler<Self, SystemStarted> for PruningPointService {
    fn handle_event(&mut self, _: SystemStarted, ctx: &mut ServiceContext<Self>) {
        ctx.notify(PruningPointInfoGeneration);
    }
}

impl EventHandler<Self, PruningPointInfoGeneration> for PruningPointService {
    fn handle_event(&mut self, _: PruningPointInfoGeneration, ctx: &mut ServiceContext<Self>) {
        let pruning_point_receiver = self.pruning_channel.pruning_receiver.clone();
        let storage = self.storage.clone();
        let dag = self.dag.clone();
        let genesis_id = self.genesis_id;
        let self_ref = ctx.self_ref();
        ctx.spawn(async move {
            match pruning_point_receiver.try_recv() {
                std::result::Result::Ok(new_dag_block) => {
                    let block_header = new_dag_block.block_header;
                    let chain_state = ChainStateDB::new(
                        storage.clone().into_super_arc(),
                        Some(block_header.state_root()),
                    );
                    let reader = AccountStateReader::new(&chain_state);
                    let FlexiDagConfigV2 {
                        pruning_depth,
                        pruning_finality,
                    } = reader
                        .get_dag_config()
                        .unwrap_or_default()
                        .unwrap_or_default();

                    match dag
                        .generate_pruning_point(
                            &block_header,
                            pruning_depth,
                            pruning_finality,
                            genesis_id,
                        )
                        .await
                    {
                        std::result::Result::Ok(_) => (),
                        Err(e) => warn!("failed to generate pruning point, error: {:?}", e),
                    }
                }
                Err(e) => match e {
                    crossbeam::channel::TryRecvError::Empty => (),
                    crossbeam::channel::TryRecvError::Disconnected => {
                        info!("pruning point receiver disconnected")
                    }
                },
            }
            match self_ref.notify(PruningPointInfoGeneration) {
                std::result::Result::Ok(_) => (),
                Err(e) => error!(
                    "failed to notify pruning point info generation, error: {:?}",
                    e
                ),
            }
        });
    }
}
