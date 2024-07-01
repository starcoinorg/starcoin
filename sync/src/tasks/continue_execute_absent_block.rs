use std::sync::Arc;

use anyhow::anyhow;
use starcoin_chain_api::ExecutedBlock;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::{error, info};
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockHeader};
use stream_task::CollectorState;

use crate::store::sync_dag_store::SyncDagStore;

pub trait ContinueChainOperator {
    fn has_dag_block(&self, block_id: HashValue) -> anyhow::Result<bool>;
    fn apply(&mut self, block: Block) -> anyhow::Result<ExecutedBlock>;
    fn notify(&mut self, executed_block: ExecutedBlock) -> anyhow::Result<CollectorState>;
}

pub struct ContinueExecuteAbsentBlock<'a> {
    operator: &'a mut dyn ContinueChainOperator,
    local_store: Arc<dyn Store>,
    sync_dag_store: SyncDagStore,
}

impl<'a> ContinueExecuteAbsentBlock<'a> {
    pub fn new(
        operator: &'a mut dyn ContinueChainOperator,
        local_store: Arc<dyn Store>,
        sync_dag_store: SyncDagStore,
    ) -> anyhow::Result<ContinueExecuteAbsentBlock<'a>> {
        anyhow::Result::Ok(ContinueExecuteAbsentBlock {
            operator,
            local_store,
            sync_dag_store,
        })
    }

    fn execute_if_parent_ready_norecursion(&mut self, parent_id: HashValue) -> anyhow::Result<()> {
        let mut parent_block_ids = vec![parent_id];

        while !parent_block_ids.is_empty() {
            let mut next_parent_blocks = vec![];
            for parent_block_id in parent_block_ids {
                let parent_block = self
                    .local_store
                    .get_dag_sync_block(parent_block_id)?
                    .ok_or_else(|| {
                        anyhow!(
                        "the dag block should exist in local store, parent child block id: {:?}",
                        parent_id,
                    )
                    })?;
                let mut executed_children = vec![];
                for child in &parent_block.children {
                    let child_block =
                        self.local_store
                            .get_dag_sync_block(*child)?
                            .ok_or_else(|| {
                                anyhow!(
                                "the dag block should exist in local store, child block id: {:?}",
                                child
                            )
                            })?;
                    if child_block
                        .block
                        .header()
                        .parents_hash()
                        .ok_or_else(|| anyhow!("the dag block's parents should exist"))?
                        .iter()
                        .all(|parent| match self.operator.has_dag_block(*parent) {
                            Ok(has) => has,
                            Err(e) => {
                                error!(
                                    "failed to get the block from the chain, block id: {:?}, error: {:?}",
                                    *parent, e
                                );
                                false
                            }
                        })
                    {
                        let executed_block = self.operator.apply(child_block.block.clone())?;
                        info!(
                            "succeed to apply a dag block: {:?}, number: {:?}",
                            executed_block.block.id(),
                            executed_block.block.header().number()
                        );
                        executed_children.push(*child);
                        self.operator.notify(executed_block)?;
                        next_parent_blocks.push(*child);
                    }
                }
                self.local_store.delete_dag_sync_block(parent_block_id)?;
                self.sync_dag_store
                    .delete_dag_sync_block(parent_block.block.header().number(), parent_block_id)?;
            }

            parent_block_ids = next_parent_blocks;
        }

        Ok(())
    }

    fn check_parents_exist(&self, block_header: &BlockHeader) -> anyhow::Result<bool> {
        let mut result = Ok(true);
        for parent in block_header.parents_hash().ok_or_else(|| {
            anyhow!(
                "the dag block's parents should exist, block id: {:?}, number: {:?}",
                block_header.id(),
                block_header.number()
            )
        })? {
            if !self.operator.has_dag_block(parent)? {
                info!("block: {:?}, number: {:?}, its parent({:?}) still dose not exist, waiting for next round", block_header.id(), block_header.number(), parent);
                let mut parent_block = self.local_store.get_dag_sync_block(parent)?.ok_or_else(|| {
                    anyhow!(
                        "the dag block should exist in local store, parent block id: {:?}, number: {:?}",
                        block_header.id(),
                        block_header.number()
                    )
                })?;
                parent_block.children.push(block_header.id());
                self.local_store.save_dag_sync_block(parent_block)?;
                result = Ok(false);
            }
        }
        result
    }

    pub fn execute_absent_blocks(
        &'a mut self,
        absent_ancestor: &mut Vec<Block>,
    ) -> anyhow::Result<()> {
        if absent_ancestor.is_empty() {
            return anyhow::Result::Ok(());
        }
        // let mut process_dag_ancestors = HashMap::new();
        let mut max_loop_count = absent_ancestor.len();
        loop {
            absent_ancestor.retain(|block| {
                match self.operator.has_dag_block(block.header().id()) {
                    Ok(has) => {
                        if has {
                            info!("{:?} was already applied", block.header().id());
                            false // remove the executed block
                        } else {
                            true // retain the un-executed block
                        }
                    }
                    Err(_) => true, // retain the un-executed block
                }
            });

            let result: anyhow::Result<()> = absent_ancestor.iter().try_for_each(|block| {
                if self.check_parents_exist(block.header())? {
                    info!(
                        "now apply for sync after fetching a dag block: {:?}, number: {:?}",
                        block.id(),
                        block.header().number()
                    );
                    let executed_block = self.operator.apply(block.clone())?;
                    info!(
                        "succeed to apply a dag block: {:?}, number: {:?}",
                        executed_block.block.id(),
                        executed_block.block.header().number()
                    );

                    self.execute_if_parent_ready_norecursion(executed_block.block.id())?;

                    self.local_store
                        .delete_dag_sync_block(executed_block.block.id())?;

                    self.sync_dag_store.delete_dag_sync_block(
                        executed_block.block.header().number(),
                        executed_block.block.id(),
                    )?;

                    self.operator.notify(executed_block)?;
                }
                anyhow::Result::Ok(())
            });
            result?;

            max_loop_count = max_loop_count.saturating_sub(1);
            if max_loop_count == 0 {
                break;
            }
        }
        Ok(())
    }
}
