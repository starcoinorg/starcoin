use chain::ChainActorRef;
use crypto::HashValue;
use logger::prelude::*;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use traits::{is_ok, ChainAsyncService, ConnectBlockError, Consensus};
use types::block::{Block, BlockInfo, BlockNumber};

struct FutureBlockPool {
    child: Arc<RwLock<HashMap<HashValue, HashSet<HashValue>>>>,
    blocks: Arc<RwLock<HashMap<HashValue, (Block, Option<BlockInfo>)>>>,
}

impl FutureBlockPool {
    pub fn new() -> Self {
        FutureBlockPool {
            child: Arc::new(RwLock::new(HashMap::new())),
            blocks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_future_block(&self, block: Block, block_info: Option<BlockInfo>) {
        let block_id = block.header().id();
        let parent_id = block.header().parent_hash();
        if !self.blocks.read().contains_key(&block_id) {
            self.blocks.write().insert(block_id, (block, block_info));
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

    pub fn take_child(&self, parent_id: &HashValue) -> Option<Vec<(Block, Option<BlockInfo>)>> {
        let descendants = self.descendants(parent_id);
        if !descendants.is_empty() {
            let mut child = Vec::new();
            let mut child_lock = self.child.write();
            let mut block_lock = self.blocks.write();
            descendants.iter().for_each(|id| {
                let _ = child_lock.remove(id);
                if let Some((block, block_info)) = block_lock.remove(id) {
                    child.push((block, block_info));
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
    pivot: Arc<RwLock<Option<BlockNumber>>>,
}

impl<C> BlockConnector<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(chain_reader: ChainActorRef<C>) -> Self {
        let pivot: Option<BlockNumber> = None;
        BlockConnector {
            chain_reader,
            future_blocks: FutureBlockPool::new(),
            pivot: Arc::new(RwLock::new(pivot)),
        }
    }

    pub fn update_pivot(&self, pivot: Option<BlockNumber>) {
        match pivot {
            Some(p) => self.pivot.write().replace(p),
            None => self.pivot.write().take(),
        };
    }

    fn get_pivot(&self) -> Option<BlockNumber> {
        *self.pivot.read()
    }

    pub async fn do_block_and_child(&self, block: Block, block_info: Option<BlockInfo>) {
        let block_id = block.header().id();
        if self.do_block_connect(block, block_info).await {
            if let Some(child) = self.future_blocks.take_child(&block_id) {
                for (son_block, son_block_info) in child {
                    let _ = self.do_block_connect(son_block, son_block_info).await;
                }
            }
        }
    }

    async fn do_block_connect(&self, block: Block, block_info: Option<BlockInfo>) -> bool {
        let pivot = self.get_pivot();
        let connect_result = if pivot.is_none() {
            self.chain_reader.clone().try_connect(block.clone()).await
        } else {
            let pivot_number = pivot.expect("pivot is none.");
            if pivot_number >= block.header().number() {
                match block_info.clone() {
                    Some(info) => {
                        self.chain_reader
                            .clone()
                            .try_connect_with_block_info(block.clone(), info)
                            .await
                    }
                    None => return false,
                }
            } else {
                self.chain_reader.clone().try_connect(block.clone()).await
            }
        };

        let block_id = block.id();
        match connect_result {
            Ok(connect) => {
                if is_ok(&connect) {
                    return true;
                } else if let Err(err) = connect {
                    match err {
                        ConnectBlockError::FutureBlock => {
                            self.future_blocks.add_future_block(block, block_info)
                        }
                        ConnectBlockError::VerifyFailed => {
                            error!("Connect block {:?} verify failed.", block_id)
                        }
                        _ => debug!("Connect block {:?} failed, because : {:?}", block_id, err),
                    }
                }
            }
            Err(e) => error!("Connect block {:?} failed : {:?}", block_id, e),
        }

        false
    }
}
