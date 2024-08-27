use std::sync::Arc;

use starcoin_chain::{verifier::DagVerifier, BlockChain, ChainReader};
use starcoin_config::TimeService;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_executor::VMMetrics;
use starcoin_logger::prelude::{error, info};
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time::{timeout, Duration},
};

#[derive(Debug)]
pub enum ExecuteState {
    Ready(HashValue),
    Executing(HashValue),
    Executed(BlockHeader),
    Error(BlockHeader),
    Closed,
}

pub struct DagBlockExecutor {
    sender: Sender<ExecuteState>,
    receiver: Receiver<Block>,
    chain: BlockChain,
}

impl DagBlockExecutor {
    pub fn new(
        sender_to_main: Sender<ExecuteState>,
        buffer_size: usize,
        time_service: Arc<dyn TimeService>,
        head_block_hash: HashValue,
        storage: Arc<dyn Store>,
        vm_metrics: Option<VMMetrics>,
        dag: BlockDAG,
    ) -> anyhow::Result<(Sender<Block>, Self)> {
        let (sender_for_main, receiver) = mpsc::channel::<Block>(buffer_size);
        let chain = BlockChain::new(time_service, head_block_hash, storage, vm_metrics, dag)?;
        let executor = Self {
            sender: sender_to_main,
            receiver,
            chain,
        };
        anyhow::Ok((sender_for_main, executor))
    }

    pub fn waiting_for_parents(
        chain: &BlockChain,
        parents_hash: Vec<HashValue>,
    ) -> anyhow::Result<bool> {
        for parent_id in parents_hash {
            if !chain.dag().has_dag_block(parent_id)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn start_to_execute(mut self) -> anyhow::Result<()> {
        tokio::spawn(async move {
            loop {
                match timeout(Duration::from_secs(10), self.receiver.recv()).await {
                    Ok(Some(block)) => {
                        let header = block.header().clone();

                        match self
                            .sender
                            .send(ExecuteState::Executing(header.id()))
                            .await
                        {
                            Ok(_) => (),
                            Err(e) => {
                                error!(
                                    "failed to send executing state: {:?}, for reason: {:?}",
                                    header, e
                                );
                                return;
                            }
                        }

                        loop {
                            match Self::waiting_for_parents(
                                &self.chain,
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

                        if self.chain.status().head().id() != block.header().parent_hash() {
                            self.chain = match self.chain.fork(block.header().parent_hash()) {
                                Ok(chain) => chain,
                                Err(e) => {
                                    error!("failed to fork in parallel for: {:?}", e);
                                    return;
                                }
                            }
                        }

                        match self.chain.apply_with_verifier::<DagVerifier>(block) {
                            Ok(executed_block) => {
                                let header = executed_block.header();
                                info!(
                                    "succeed to execute block: number: {:?}, id: {:?}",
                                    header.number(),
                                    header.id()
                                );
                                match self
                                    .sender
                                    .send(ExecuteState::Executed(header.clone()))
                                    .await
                                {
                                    Ok(_) => (),
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
                    Ok(None) => {
                        info!("sync worker channel closed");
                        return;
                    }
                    Err(e) => {
                        info!("timeout occurs: {:?}", e);
                        return;
                    }
                }
            }
        });

        anyhow::Ok(())
    }
}
