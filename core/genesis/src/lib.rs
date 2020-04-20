// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use starcoin_accumulator::node::ACCUMULATOR_PLACEHOLDER_HASH;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_config::{ChainNetwork, VMConfig};
use starcoin_consensus::{argon::ArgonConsensus, dummy::DummyConsensus};
use starcoin_crypto::{hash::CryptoHash, HashValue};
use starcoin_executor::executor::Executor;
use starcoin_executor::TransactionExecutor;
use starcoin_logger::prelude::*;
use starcoin_state_api::ChainState;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{Storage, Store};
use starcoin_types::block::BlockInfo;
use starcoin_types::startup_info::{ChainInfo, StartupInfo};
use starcoin_types::state_set::ChainStateSet;
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::{block::Block, transaction::Transaction, vm_error::StatusCode, U512};
use std::fmt::Display;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use traits::Consensus;

pub static GENESIS_FILE_NAME: &str = "genesis";

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Genesis {
    state: ChainStateSet,
    block: Block,
}

impl Display for Genesis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Genesis {{")?;
        write!(f, "state: {{ len={} }}, ", self.state.len())?;
        write!(f, "block: {:?}", self.block)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl Genesis {
    pub fn build(net: ChainNetwork) -> Result<Self> {
        match net {
            ChainNetwork::Dev => Self::do_build::<DummyConsensus>(ChainNetwork::Dev),
            net => Self::do_build::<ArgonConsensus>(net),
        }
    }

    fn do_build<C>(net: ChainNetwork) -> Result<Self>
    where
        C: Consensus + 'static,
    {
        debug!("Init genesis");
        let chain_config = net.get_config();
        //TODO remove config argument, genesis not dependency NodeConfig.
        let (_state_root, chain_state_set) = Executor::init_genesis(&chain_config)?;

        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance(
            CacheStorage::new(),
        ))?);
        let chain_state_db = ChainStateDB::new(storage.clone(), None);

        let transaction_info = Self::execute_genesis_txn(chain_state_set.clone(), &chain_state_db)?;

        let accumulator = MerkleAccumulator::new(
            HashValue::zero(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            0,
            storage.clone(),
        )?;
        let txn_info_hash = transaction_info.crypto_hash();

        let (accumulator_root, _) = accumulator.append(vec![txn_info_hash].as_slice())?;

        let block = Block::genesis_block(
            accumulator_root,
            transaction_info.state_root_hash(),
            chain_config.difficult,
            chain_config.consensus_header.clone(),
        );
        assert_eq!(block.header().number(), 0);
        debug!("Genesis block id : {:?}", block.header().id());

        let genesis = Self {
            state: chain_state_set,
            block,
        };
        Ok(genesis)
    }

    fn execute_genesis_txn(
        chain_state_set: ChainStateSet,
        chain_state: &dyn ChainState,
    ) -> Result<TransactionInfo> {
        let txn = Transaction::StateSet(chain_state_set);
        let txn_hash = txn.crypto_hash();

        let vm_config = VMConfig::default();
        let output = Executor::execute_transaction(&vm_config, chain_state, txn)?;
        ensure!(
            output.status().vm_status().major_status == StatusCode::EXECUTED,
            "Genesis txn execute fail."
        );
        let state_root = chain_state.commit()?;

        Ok(TransactionInfo::new(
            txn_hash,
            state_root,
            //TODO genesis event.
            HashValue::zero(),
            0,
            output.status().vm_status().major_status,
        ))
    }

    pub fn state(&self) -> &ChainStateSet {
        &self.state
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn load<P>(data_dir: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        let genesis_file_path = data_dir.as_ref().join(GENESIS_FILE_NAME);
        if !genesis_file_path.exists() {
            return Ok(None);
        }
        let mut genesis_file = File::open(genesis_file_path)?;
        let mut content = vec![];
        genesis_file.read_to_end(&mut content)?;
        let genesis = scs::from_bytes(&content)?;
        return Ok(Some(genesis));
    }

    pub fn execute(self, storage: Arc<dyn Store>) -> Result<StartupInfo> {
        let Genesis { state, block } = self;

        let chain_state_db = ChainStateDB::new(storage.clone().into_super_arc(), None);
        let transaction_info = Self::execute_genesis_txn(state, &chain_state_db)?;

        ensure!(
            block.header().state_root() == transaction_info.state_root_hash(),
            "Genesis block state root mismatch."
        );

        let accumulator = MerkleAccumulator::new(
            block.header().id(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            0,
            storage.clone().into_super_arc(),
        )?;
        let txn_info_hash = transaction_info.crypto_hash();

        let (accumulator_root, _) = accumulator.append(vec![txn_info_hash].as_slice())?;

        ensure!(
            block.header().number() == 0,
            "Genesis block number must is 0."
        );
        debug!("Genesis block id : {:?}", block.header().id());

        ensure!(
            block.header().accumulator_root() == accumulator_root,
            "Genesis block accumulator root mismatch."
        );
        //TODO verify consensus header
        let chain_info = ChainInfo::new(None, block.header().id(), block.header());
        storage.commit_branch_block(block.header().id(), block.clone())?;

        let startup_info = StartupInfo::new(chain_info, vec![]);

        //save block info for accumulator init
        storage.save_block_info(BlockInfo::new(
            block.header().id(),
            accumulator_root,
            accumulator.get_frozen_subtree_roots().unwrap(),
            accumulator.num_leaves(),
            accumulator.num_nodes(),
            U512::zero(),
        ))?;
        storage.save_startup_info(startup_info.clone())?;
        Ok(startup_info)
    }

    pub fn save<P>(&self, data_dir: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let data_dir = data_dir.as_ref();
        if !data_dir.exists() {
            create_dir_all(data_dir)?;
        }
        let genesis_file = data_dir.join(GENESIS_FILE_NAME);
        let mut file = File::create(genesis_file)?;
        let contents = scs::to_bytes(self)?;
        file.write_all(&contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_storage::cache_storage::CacheStorage;
    use starcoin_storage::storage::StorageInstance;
    use starcoin_storage::Storage;

    #[stest::test]
    pub fn test_genesis() -> Result<()> {
        for net in ChainNetwork::networks() {
            do_test_genesis(net)?;
        }
        Ok(())
    }

    pub fn do_test_genesis(net: ChainNetwork) -> Result<()> {
        let temp_dir = starcoin_config::temp_path();
        let genesis = Genesis::build(net)?;
        genesis.save(temp_dir.as_ref())?;
        let genesis2 = Genesis::load(temp_dir.as_ref())?;
        assert!(genesis2.is_some(), "load genesis fail.");
        let genesis2 = genesis2.unwrap();
        assert_eq!(genesis, genesis2, "genesis save and load different.");

        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance(
            CacheStorage::new(),
        ))?);
        let startup_info = genesis.execute(storage)?;

        let storage2 = Arc::new(Storage::new(StorageInstance::new_cache_instance(
            CacheStorage::new(),
        ))?);
        let startup_info2 = genesis2.execute(storage2)?;

        assert_eq!(
            startup_info, startup_info2,
            "genesis execute startup info different."
        );
        Ok(())
    }
}
