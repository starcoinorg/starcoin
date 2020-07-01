use crate::state_sync::StateSyncTaskRef;
use chain::ChainActorRef;
use crypto::HashValue;
use logger::prelude::*;
use parking_lot::RwLock;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_storage::Store;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use traits::{ChainAsyncService, ConnectBlockResult, Consensus};
use types::block::{Block, BlockInfo, BlockNumber};

#[derive(Clone)]
pub struct PivotBlock<C>
where
    C: Consensus + 'static,
{
    number: BlockNumber,
    block_info: BlockInfo,
    state_sync_task_ref: StateSyncTaskRef<C>,
    block_accumulator: Option<Arc<MerkleAccumulator>>,
    storage: Arc<dyn Store>,
}

impl<C> PivotBlock<C>
where
    C: Consensus,
{
    pub fn new(
        number: BlockNumber,
        block_info: BlockInfo,
        state_sync_task_ref: StateSyncTaskRef<C>,
        storage: Arc<dyn Store>,
    ) -> Self {
        Self {
            number,
            block_info,
            state_sync_task_ref,
            block_accumulator: None,
            storage,
        }
    }
}

struct FutureBlockPool {
    child: Arc<RwLock<HashMap<HashValue, HashSet<HashValue>>>>,
    blocks: Arc<RwLock<HashMap<HashValue, Block>>>,
}

impl FutureBlockPool {
    pub fn new() -> Self {
        FutureBlockPool {
            child: Arc::new(RwLock::new(HashMap::new())),
            blocks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_future_block(&self, block: Block) {
        let block_id = block.header().id();
        let parent_id = block.header().parent_hash();
        if !self.blocks.read().contains_key(&block_id) {
            self.blocks.write().insert(block_id, block);
        }
        let mut lock = self.child.write();
        if lock.contains_key(&parent_id) {
            lock.get_mut(&parent_id)
                .expect("parent not exist.")
                .insert(block_id);
        } else {
            let mut child = HashSet::new();
            child.insert(block_id);
            lock.insert(parent_id, child);
        }
    }

    fn descendants(&self, parent_id: &HashValue) -> Vec<HashValue> {
        let mut child = Vec::new();
        let lock = self.child.read();
        if lock.contains_key(parent_id) {
            lock.get(parent_id)
                .expect("parent not exist.")
                .iter()
                .for_each(|id| {
                    child.push(*id);
                });

            if !child.is_empty() {
                child.clone().iter().for_each(|new_parent_id| {
                    let mut new_child = self.descendants(new_parent_id);
                    if !new_child.is_empty() {
                        child.append(&mut new_child);
                    }
                })
            }
        };

        child
    }

    pub fn take_child(&self, parent_id: &HashValue) -> Option<Vec<Block>> {
        let descendants = self.descendants(parent_id);
        if !descendants.is_empty() {
            let mut child = Vec::new();
            let mut child_lock = self.child.write();
            let mut block_lock = self.blocks.write();
            descendants.iter().for_each(|id| {
                let _ = child_lock.remove(id);
                if let Some(block) = block_lock.remove(id) {
                    child.push(block);
                }
            });
            Some(child)
        } else {
            None
        }
    }
}

pub struct BlockConnector<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    chain_reader: ChainActorRef<C>,
    future_blocks: FutureBlockPool,
    pivot: Arc<RwLock<Option<PivotBlock<C>>>>,
}

impl<C> BlockConnector<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(chain_reader: ChainActorRef<C>) -> Self {
        let pivot: Option<PivotBlock<C>> = None;
        BlockConnector {
            chain_reader,
            future_blocks: FutureBlockPool::new(),
            pivot: Arc::new(RwLock::new(pivot)),
        }
    }

    pub fn update_pivot(&self, pivot: Option<PivotBlock<C>>) {
        match pivot {
            Some(p) => self.pivot.write().replace(p),
            None => self.pivot.write().take(),
        };
    }

    fn get_pivot(&self) -> Option<PivotBlock<C>> {
        (*self.pivot.read()).clone()
    }

    fn get_block_accumulator(&self) -> Option<Arc<MerkleAccumulator>> {
        let mut lock = self.pivot.write();
        let lock = lock.as_mut();
        lock.and_then(|pivot_block| -> Option<Arc<MerkleAccumulator>> {
            let block_accumulator_info = pivot_block.block_info.get_block_accumulator_info();
            if pivot_block.block_accumulator.is_none() {
                let block_accumulator = MerkleAccumulator::new(
                    *block_accumulator_info.get_accumulator_root(),
                    block_accumulator_info
                        .get_frozen_subtree_roots()
                        .clone()
                        .to_vec(),
                    block_accumulator_info.get_num_leaves(),
                    block_accumulator_info.get_num_nodes(),
                    AccumulatorStoreType::Block,
                    pivot_block.storage.clone().into_super_arc(),
                )
                .unwrap();
                pivot_block.block_accumulator = Some(Arc::new(block_accumulator));
            }
            Some(pivot_block.block_accumulator.clone().unwrap())
        })
    }

    pub async fn do_block_and_child(&self, block: Block) {
        let block_id = block.header().id();
        if self.do_block_connect(block).await {
            if let Some(child) = self.future_blocks.take_child(&block_id) {
                for son_block in child {
                    let _ = self.do_block_connect(son_block).await;
                }
            }
        }
    }

    async fn do_block_connect(&self, block: Block) -> bool {
        let pivot = self.get_pivot();
        let mut _state_sync_address = None;
        let connect_result = if pivot.is_none() {
            self.chain_reader.clone().try_connect(block.clone()).await
        } else {
            let tmp = pivot.expect("pivot is none.");
            let pivot_number = tmp.number;
            _state_sync_address = Some(tmp.state_sync_task_ref);
            let number = block.header().number();
            if pivot_number >= number {
                let block_accumulator = self
                    .get_block_accumulator()
                    .expect("Get block accumulator failed.");
                match block_accumulator.get_leaf(number) {
                    Ok(Some(block_id)) => {
                        let current_block_id = block.id();
                        if block_id == current_block_id {
                            self.chain_reader
                                .clone()
                                .try_connect_without_execute(block.clone())
                                .await
                        } else {
                            error!("block miss match {:?} : {:?}", block_id, current_block_id);
                            Ok(ConnectBlockResult::VerifyBlockIdFailed)
                        }
                    }
                    Ok(None) => Ok(ConnectBlockResult::VerifyBlockIdFailed),
                    Err(err) => {
                        error!("Get block accumulator leaf {:?} failed : {:?}", number, err);
                        Ok(ConnectBlockResult::VerifyBlockIdFailed)
                    }
                }
            } else {
                self.chain_reader.clone().try_connect(block.clone()).await
            }
        };

        let block_id = block.id();
        match connect_result {
            Ok(connect) => {
                match connect {
                    ConnectBlockResult::SUCCESS | ConnectBlockResult::DuplicateConn => {
                        return true;
                    }
                    ConnectBlockResult::FutureBlock => self.future_blocks.add_future_block(block),
                    ConnectBlockResult::VerifyBlockIdFailed => {
                        //TODO
                    }
                    ConnectBlockResult::VerifyConsensusFailed => {
                        error!("Connect block {:?} verify nonce failed.", block_id);
                        //TODO: remove child block
                    }
                    ConnectBlockResult::VerifyBodyFailed => {
                        error!("Connect block {:?} verify body failed.", block_id);
                        //TODO:
                    }
                    ConnectBlockResult::VerifyTxnInfoFailed => {
                        error!("Connect block {:?} verify txn info failed.", block_id);
                        //todo: state_sync_address.expect("").reset();
                    }
                }
            }
            Err(e) => error!("Connect block {:?} failed : {:?}", block_id, e),
        }

        false
    }
}
