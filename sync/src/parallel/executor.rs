use std::sync::Arc;

use parking_lot::RwLock;
use starcoin_chain::{verifier::{DagVerifier, DagVerifierWithGhostData}, BlockChain, ChainReader};
use starcoin_chain_api::ExecutedBlock;
use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{error, info};
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader};
use tokio::{
    sync::mpsc::{self, Receiver, Sender}, task::JoinHandle, time::{timeout, Duration}
};

use crate::tasks::continue_execute_absent_block::ContinueChainOperator;

#[derive(Debug)]
pub enum ExecuteState {
    Executing(HashValue),
    Executed(ExecutedBlock),
    Error(BlockHeader),
    Closed,
}

pub struct DagBlockExecutor {
    sender: Sender<ExecuteState>,
    receiver: Receiver<Block>,
    time_service: Arc<dyn TimeService>,
    storage: Arc<dyn Store>,
    vm_metrics: Option<VMMetrics>,
    dag: BlockDAG,
}

impl DagBlockExecutor {
    pub fn new(
        sender_to_main: Sender<ExecuteState>,
        buffer_size: usize,
        time_service: Arc<dyn TimeService>,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
    ) -> anyhow::Result<(Sender<Block>, Self)> {
        let (sender_for_main, receiver) = mpsc::channel::<Block>(buffer_size);
        let executor = Self {
            sender: sender_to_main,
            receiver,
            time_service,
            storage,
            vm_metrics,
            dag,
        };
        anyhow::Ok((sender_for_main, executor))
    }

    pub fn waiting_for_parents(
        chain: &BlockDAG,
        parents_hash: Vec<HashValue>,
    ) -> anyhow::Result<bool> {
        for parent_id in parents_hash {
            if !chain.has_dag_block(parent_id)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn start_to_execute(mut self) -> anyhow::Result<JoinHandle<()>> {
        let handle = tokio::spawn(async move {
            let mut chain = None;
            loop {
                match self.receiver.recv().await {
                    Some(block) => {
                        let header = block.header().clone();

                        loop {
                            match Self::waiting_for_parents(
                                &self.dag,
                                block.header().parents_hash(),
                            ) {
                                Ok(true) => break,
                                Ok(false) => tokio::task::yield_now().await,
                                Err(e) => {
                                    error!(
                                        "failed to check parents: {:?}, for reason: {:?}",
                                        header, e
                                    );
                                    match self
                                        .sender
                                        .send(ExecuteState::Error(header.clone()))
                                        .await
                                    {
                                        Ok(_) => (),
                                        Err(e) => {
                                            error!("failed to send error state: {:?}, for reason: {:?}", header, e);
                                            return;
                                        }
                                    }
                                    return;
                                }
                            }
                        }

                        match chain {
                            None => {
                                chain = match BlockChain::new(self.time_service.clone(), block.header().parent_hash(), self.storage.clone(), self.vm_metrics.clone(), self.dag.clone()) {
                                    Ok(new_chain) => Some(new_chain),
                                    Err(e) => {
                                        error!("failed to create chain for block: {:?} for {:?}", block.header().id(), e);
                                        return;
                                    }
                                }
                            }
                            Some(old_chain) => {
                                if old_chain.status().head().id() != block.header().parent_hash(){
                                    chain = match old_chain.fork(block.header().parent_hash()) {
                                        Ok(new_chain) => Some(new_chain),
                                        Err(e) => {
                                            error!("failed to fork in parallel for: {:?}", e);
                                            return;
                                        }
                                    }
                                } else {
                                    chain = Some(old_chain);
                                }
                            }
                        }

                       info!("sync parallel worker {:p} will execute block: {:?}", &self, block.header().id());
                        match chain.as_mut().expect("it cannot be none!").apply_with_verifier::<DagVerifierWithGhostData>(block) {
                            Ok(executed_block) => {
                                info!(
                                    "succeed to execute block: number: {:?}, id: {:?}",
                                    executed_block.header().number(),
                                    executed_block.header().id()
                                );
                                match self
                                    .sender
                                    .send(ExecuteState::Executed(executed_block))
                                    .await
                                {
                                    Ok(_) => tokio::task::yield_now().await,
                                    Err(e) => {
                                        error!(
                                            "failed to send waiting state: {:?}, for reason: {:?}",
                                            header, e
                                        );
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                error!(
                                    "failed to execute block: {:?}, for reason: {:?}",
                                    header, e
                                );
                                match self.sender.send(ExecuteState::Error(header.clone())).await {
                                    Ok(_) => (),
                                    Err(e) => {
                                        error!(
                                            "failed to send error state: {:?}, for reason: {:?}",
                                            header, e
                                        );
                                        return;
                                    }
                                }
                                return;
                            }
                        }
                    }
                    None => {
                        info!("sync worker channel closed");
                        drop(self.sender);
                        return;
                    }
                }
            }
        });

        anyhow::Ok(handle)
    }
}
