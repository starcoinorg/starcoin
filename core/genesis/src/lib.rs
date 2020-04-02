// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_config::NodeConfig;
use starcoin_consensus::Consensus;
use starcoin_crypto::{hash::CryptoHash, HashValue};
use starcoin_executor::TransactionExecutor;
use starcoin_logger::prelude::*;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{BlockStore, Store};
use starcoin_types::block::BlockInfo;
use starcoin_types::startup_info::{ChainInfo, StartupInfo};
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::{block::Block, transaction::Transaction};
use std::sync::Arc;

#[derive(Debug)]
pub struct Genesis {
    transaction: Transaction,
    transaction_info: TransactionInfo,
    block: Block,
    startup_info: StartupInfo,
}

impl Genesis {
    pub fn new<E, C, S>(config: Arc<NodeConfig>, storage: Arc<S>) -> Result<Self>
    where
        E: TransactionExecutor + 'static,
        C: Consensus + 'static,
        S: Store + 'static,
    {
        info!("Init genesis");
        //TODO init genesis by network
        let (state_root, chain_state_set) = E::init_genesis(&config.vm)?;
        let chain_state_db = ChainStateDB::new(storage.clone(), None);
        let transaction = Transaction::StateSet(chain_state_set);
        let output = E::execute_transaction(&config.vm, &chain_state_db, transaction.clone())?;
        let txn_hash = transaction.crypto_hash();
        let transaction_info = TransactionInfo::new(
            txn_hash,
            state_root,
            HashValue::zero(),
            output.gas_used(),
            output.status().vm_status().major_status,
        );
        let accumulator = MerkleAccumulator::new(vec![], 0, 0, storage.clone())?;
        let txn_info_hash = transaction_info.crypto_hash();
        let (accumulator_root, _) = accumulator.append(vec![txn_info_hash].as_slice())?;
        let consensus_header = C::init_genesis_header(config.clone());
        let block = Block::genesis_block(accumulator_root, state_root, consensus_header);
        assert_eq!(block.header().number(), 0);
        info!("Genesis block id : {:?}", block.header().id());
        let chain_info = ChainInfo::new(None, block.header().id(), block.header());
        BlockStore::commit_branch_block(storage.as_ref(), block.header().id(), block.clone())?;
        let startup_info = StartupInfo::new(chain_info, vec![]);
        //save block info for accumulator init
        storage.save_block_info(BlockInfo::new(
            block.header().id(),
            accumulator.get_frozen_subtree_roots().unwrap(),
            accumulator.num_leaves(),
            accumulator.num_nodes(),
        ))?;
        let _ = storage.save_startup_info(startup_info.clone());
        info!("Genesis startup info: {:?}", startup_info);
        Ok(Self {
            transaction,
            transaction_info,
            block,
            startup_info,
        })
    }

    pub fn startup_info(&self) -> &StartupInfo {
        &self.startup_info
    }

    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    pub fn transaction_info(&self) -> &TransactionInfo {
        &self.transaction_info
    }

    pub fn block(&self) -> &Block {
        &self.block
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_consensus::dummy::DummyConsensus;
    use starcoin_executor::executor::Executor;
    use starcoin_storage::cache_storage::CacheStorage;
    use starcoin_storage::db_storage::DBStorage;
    use starcoin_storage::storage::StorageInstance;
    use starcoin_storage::Storage;

    #[stest::test]
    pub fn test_genesis() -> Result<()> {
        let config = Arc::new(NodeConfig::default());
        let cache_storage = Arc::new(CacheStorage::new());
        let tmpdir = libra_temppath::TempPath::new();
        let db_storage = Arc::new(DBStorage::new(tmpdir.path()));
        let storage = Arc::new(
            Storage::new(StorageInstance::new_cache_and_db_instance(
                cache_storage.clone(),
                db_storage.clone(),
            ))
            .unwrap(),
        );
        let genesis = Genesis::new::<Executor, DummyConsensus, Storage>(config, storage)
            .expect("init genesis must success.");
        info!("genesis: {:?}", genesis);
        Ok(())
    }
}
