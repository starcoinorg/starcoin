use std::sync::Arc;

use crate::blockdag::BlockDAG;
use anyhow::format_err;
use crossbeam::channel::{self, Receiver, Sender};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::{error, info, warn};
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_types::{block::BlockHeader, system_events::SystemStarted};
use starcoin_vm2_storage::Store as Store2;

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
    storage2: Arc<dyn Store2>,
}

impl PruningPointService {
    pub fn new(
        dag: BlockDAG,
        pruning_channel: PruningPointInfoChannel,
        genesis_id: HashValue,
        storage2: Arc<dyn Store2>,
    ) -> Self {
        Self {
            dag,
            pruning_channel,
            genesis_id,
            storage2,
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
        let storage2 = ctx.get_shared::<Arc<dyn Store2>>()?;
        let genesis_id = storage
            .get_genesis()?
            .ok_or_else(|| format_err!("genesis not found"))?;
        anyhow::Ok(Self::new(dag, pruning_channel, genesis_id, storage2))
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
        let storage2 = self.storage2.clone();
        let dag = self.dag.clone();
        let genesis_id = self.genesis_id;
        let self_ref = ctx.self_ref();
        let storage = ctx.get_shared::<Arc<Storage>>().unwrap();
        ctx.spawn(async move {
            match pruning_point_receiver.try_recv() {
                std::result::Result::Ok(new_dag_block) => {
                    let block_header = new_dag_block.block_header;

                    // Get pruning config from VM2 epoch
                    use starcoin_vm2_chain::get_epoch_from_statedb;
                    use starcoin_vm2_statedb::ChainStateDB;

                    // Get the correct VM2 state root from multi_state
                    let multi_state = match storage.get_vm_multi_state(block_header.id()) {
                        Ok(ms) => ms,
                        Err(e) => {
                            error!(
                                "Failed to get multi_state for block {}: {:?}",
                                block_header.id(),
                                e
                            );
                            return;
                        }
                    };

                    let chain_state = ChainStateDB::new(
                        storage2.clone().into_super_arc(),
                        Some(multi_state.state_root2()),
                    );

                    let (pruning_depth, pruning_finality) =
                        match get_epoch_from_statedb(&chain_state) {
                            Ok(epoch) => (epoch.pruning_depth(), epoch.pruning_finality()),
                            Err(e) => {
                                error!("Failed to get epoch from VM2 state: {:?}", e);
                                return;
                            }
                        };

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
