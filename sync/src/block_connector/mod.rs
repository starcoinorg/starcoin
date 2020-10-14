// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state_sync::StateSyncTaskRef;
use anyhow::{format_err, Result};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use network_api::PeerId;
use parking_lot::RwLock;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
use starcoin_storage::Store;
use starcoin_types::{
    block::{Block, BlockInfo, BlockNumber},
    startup_info::StartupInfo,
};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use traits::WriteableChainService;
use traits::{ConnectBlockError, VerifyBlockField};
use txpool::TxPoolService;

mod block_connect_test;
mod metrics;
#[cfg(test)]
mod test_illegal_block;
#[cfg(test)]
mod test_write_block_chain;
mod write_block_chain;

use starcoin_network_rpc_api::RemoteChainStateReader;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::ServiceRef;
pub use write_block_chain::WriteBlockChainService;

#[cfg(test)]
pub use test_write_block_chain::create_writeable_block_chain;
#[cfg(test)]
pub use test_write_block_chain::gen_blocks;
#[cfg(test)]
pub use test_write_block_chain::new_block;

#[derive(Clone)]
pub struct PivotBlock {
    number: BlockNumber,
    block_info: BlockInfo,
    state_sync_task_ref: StateSyncTaskRef,
    block_accumulator: Option<Arc<MerkleAccumulator>>,
    storage: Arc<dyn Store>,
}

impl PivotBlock {
    pub fn new(
        number: BlockNumber,
        block_info: BlockInfo,
        state_sync_task_ref: StateSyncTaskRef,
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

        if let Some(set) = lock.get(parent_id) {
            set.iter().for_each(|id| {
                child.push(*id);
                let mut new_child = self.descendants(id);
                if !new_child.is_empty() {
                    child.append(&mut new_child);
                }
            })
        }

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

            let _ = child_lock.remove(parent_id);
            Some(child)
        } else {
            None
        }
    }

    pub fn _len(&self) -> usize {
        self.blocks.read().len()
    }

    pub fn _son_len(&self, block_id: &HashValue) -> usize {
        let lock = self.child.read();
        match lock.get(&block_id) {
            None => 0,
            Some(child) => child.len(),
        }
    }
}

pub struct BlockConnector {
    writeable_block_chain: Arc<RwLock<dyn WriteableChainService + 'static>>,
    future_blocks: FutureBlockPool,
    pivot: Arc<RwLock<Option<PivotBlock>>>,
}

impl BlockConnector {
    pub fn new(
        config: Arc<NodeConfig>,
        startup_info: StartupInfo,
        storage: Arc<dyn Store>,
        txpool: TxPoolService,
        bus: ServiceRef<BusService>,
        remote_chain_state: Option<RemoteChainStateReader>,
    ) -> Self {
        let pivot: Option<PivotBlock> = None;
        let writeable_block_chain = WriteBlockChainService::new(
            config,
            startup_info,
            storage,
            txpool,
            bus,
            remote_chain_state,
        )
        .unwrap();
        BlockConnector {
            writeable_block_chain: Arc::new(RwLock::new(writeable_block_chain)),
            future_blocks: FutureBlockPool::new(),
            pivot: Arc::new(RwLock::new(pivot)),
        }
    }

    pub fn update_pivot(&self, pivot: Option<PivotBlock>) {
        match pivot {
            Some(p) => self.pivot.write().replace(p),
            None => self.pivot.write().take(),
        };
    }

    fn get_pivot(&self) -> Option<PivotBlock> {
        (*self.pivot.read()).clone()
    }

    fn get_block_accumulator(&self) -> Option<Arc<MerkleAccumulator>> {
        let mut lock = self.pivot.write();
        let lock = lock.as_mut();
        lock.map(|pivot_block| -> Arc<MerkleAccumulator> {
            let block_accumulator_info = pivot_block.block_info.get_block_accumulator_info();
            if pivot_block.block_accumulator.is_none() {
                let block_accumulator = MerkleAccumulator::new_with_info(
                    block_accumulator_info.clone(),
                    pivot_block
                        .storage
                        .get_accumulator_store(AccumulatorStoreType::Block),
                );
                pivot_block.block_accumulator = Some(Arc::new(block_accumulator));
            }
            pivot_block.block_accumulator.clone().unwrap()
        })
    }

    pub fn do_block_and_child(&self, block: Block, peer_id: PeerId) {
        let block_id = block.header().id();
        if self.do_block_connect(block, peer_id.clone()) {
            if let Some(child) = self.future_blocks.take_child(&block_id) {
                for son_block in child {
                    let _ = self.do_block_connect(son_block, peer_id.clone());
                }
            }
        }
    }

    pub fn connect_block(&self, block: Block) -> Result<()> {
        self.writeable_block_chain.write().try_connect(block)
    }

    fn do_block_connect(&self, block: Block, peer_id: PeerId) -> bool {
        debug!(
            "connect begin block {:?} : {:?}",
            block.header().number(),
            block.id()
        );
        let pivot = self.get_pivot();
        let mut _state_sync_address = None;
        let current_block_id = block.id();
        let connect_result = if pivot.is_none() {
            self.writeable_block_chain
                .write()
                .try_connect(block.clone())
        } else {
            let tmp = pivot.expect("pivot is none.");
            let pivot_number = tmp.number;
            let pivot_id = tmp.block_info.block_id();
            _state_sync_address = Some(tmp.state_sync_task_ref);
            let number = block.header().number();
            match pivot_number.cmp(&number) {
                Ordering::Greater => {
                    let block_accumulator = self
                        .get_block_accumulator()
                        .expect("Get block accumulator failed.");
                    match block_accumulator.get_leaf(number) {
                        Ok(Some(block_id)) => {
                            if block_id == current_block_id {
                                self.writeable_block_chain
                                    .write()
                                    .try_connect_without_execute(block.clone(), peer_id)
                            } else {
                                Err(ConnectBlockError::VerifyBlockFailed(
                                    VerifyBlockField::Header,
                                    format_err!(
                                        "block miss match : {:?} :{:?} : {:?}",
                                        number,
                                        block_id,
                                        current_block_id
                                    ),
                                )
                                .into())
                            }
                        }
                        Ok(None) => Err(ConnectBlockError::VerifyBlockFailed(
                            VerifyBlockField::Header,
                            format_err!("Can not find block accumulator leaf {:?} failed", number,),
                        )
                        .into()),
                        Err(err) => Err(ConnectBlockError::VerifyBlockFailed(
                            VerifyBlockField::Header,
                            format_err!(
                                "Get block accumulator leaf {:?} failed : {:?}",
                                number,
                                err,
                            ),
                        )
                        .into()),
                    }
                }
                Ordering::Equal => {
                    let parent_id = block.header().parent_hash();
                    if pivot_id == &parent_id {
                        self.writeable_block_chain
                            .write()
                            .try_connect_without_execute(block.clone(), peer_id)
                    } else {
                        Err(ConnectBlockError::VerifyBlockFailed(
                            VerifyBlockField::Header,
                            format_err!(
                                "pivot block id miss match : {:?} :{:?} : {:?}",
                                number,
                                pivot_id,
                                parent_id
                            ),
                        )
                        .into())
                    }
                }
                Ordering::Less => self
                    .writeable_block_chain
                    .write()
                    .try_connect(block.clone()),
            }
        };

        match connect_result {
            Ok(_) => {
                debug!(
                    "connect succ block {:?} : {:?}",
                    block.header().number(),
                    block.id()
                );
                return true;
            }
            Err(e) => {
                match e.downcast::<ConnectBlockError>() {
                    Ok(connect_error) => {
                        match connect_error {
                            ConnectBlockError::FutureBlock(block) => {
                                //TODO use error's block.
                                self.future_blocks.add_future_block(*block)
                            }
                            err => {
                                error!("Connect block {:?}, failed: {:?}", current_block_id, err)
                            }
                        }
                    }
                    Err(err) => error!("Connect block {:?} failed : {:?}", current_block_id, err),
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::FutureBlockPool;
    use starcoin_types::block::{Block, BlockBody, BlockHeader};

    #[test]
    fn test_future_block_pool() {
        let parent_header = BlockHeader::random();
        let parent_id = parent_header.id();
        let parent_number = parent_header.number();
        let body = BlockBody::new(Vec::new(), None);
        let parent = Block::new(parent_header, body.clone());

        let mut son_header = BlockHeader::random();
        son_header.parent_hash = parent_id;
        son_header.number = parent_number + 1;
        let son = Block::new(son_header, body);

        let pool = FutureBlockPool::new();
        pool.add_future_block(son);
        let descendants = pool.descendants(&parent.id());
        assert_eq!(descendants.len(), 1, "descendants length mismatch.");
        let son_blocks = pool.take_child(&parent.id());
        assert_eq!(son_blocks.unwrap().len(), 1, "son_blocks length mismatch.");
    }
}
