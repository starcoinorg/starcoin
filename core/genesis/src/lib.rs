// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use starcoin_accumulator::node::{AccumulatorStoreType, ACCUMULATOR_PLACEHOLDER_HASH};
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_config::ChainNetwork;
use starcoin_consensus::{argon::ArgonConsensus, dev::DevConsensus};
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_executor::executor::Executor;
use starcoin_executor::TransactionExecutor;
use starcoin_logger::prelude::*;
use starcoin_state_api::ChainState;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{Storage, Store};
use starcoin_types::block::{BlockInfo, BlockState};
use starcoin_types::startup_info::StartupInfo;
use starcoin_types::transaction::{ChangeSet, TransactionInfo};
use starcoin_types::{
    accumulator_info::AccumulatorInfo, block::Block, transaction::Transaction,
    vm_error::StatusCode, U512,
};
use std::convert::TryInto;
use std::fmt::Display;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use traits::Consensus;

pub static GENESIS_FILE_NAME: &str = "genesis";

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Genesis {
    state: ChangeSet,
    block: Block,
}

impl Display for Genesis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Genesis {{")?;
        write!(f, "state: {{ len={} }}, ", self.state.write_set().len())?;
        write!(f, "block: {:?}", self.block)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl Genesis {
    pub fn build(net: ChainNetwork) -> Result<Self> {
        match net {
            ChainNetwork::Dev => Self::do_build::<DevConsensus>(ChainNetwork::Dev),
            net => Self::do_build::<ArgonConsensus>(net),
        }
    }

    fn do_build<C>(net: ChainNetwork) -> Result<Self>
    where
        C: Consensus + 'static,
    {
        debug!("Init genesis");
        let chain_config = net.get_config();
        let change_set = Executor::init_genesis(&chain_config)?;

        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance(
            CacheStorage::new(),
        ))?);
        let chain_state_db = ChainStateDB::new(storage.clone(), None);

        let transaction_info = Self::execute_genesis_txn(change_set.clone(), &chain_state_db)?;

        let accumulator = MerkleAccumulator::new(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            0,
            AccumulatorStoreType::Transaction,
            storage,
        )?;
        let txn_info_hash = transaction_info.crypto_hash();

        let (accumulator_root, _) = accumulator.append(vec![txn_info_hash].as_slice())?;
        accumulator.flush()?;
        let block = Block::genesis_block(
            accumulator_root,
            transaction_info.state_root_hash(),
            chain_config.difficulty,
            chain_config.consensus_header.clone(),
        );
        assert_eq!(block.header().number(), 0);
        debug!("Genesis block id : {:?}", block.header().id());

        let genesis = Self {
            state: change_set,
            block,
        };
        Ok(genesis)
    }

    fn execute_genesis_txn(
        change_set: ChangeSet,
        chain_state: &dyn ChainState,
    ) -> Result<TransactionInfo> {
        let txn = Transaction::ChangeSet(change_set);
        let txn_hash = txn.crypto_hash();

        let output = Executor::execute_transactions(chain_state.as_super(), vec![txn])?
            .pop()
            .expect("Execute output must exist.");
        let (write_set, events, gas_used, status) = output.into_inner();
        ensure!(
            status.vm_status().major_status == StatusCode::EXECUTED,
            "Genesis txn execute fail for: {:?}",
            status
        );
        chain_state.apply_write_set(write_set)?;
        let state_root = chain_state.commit()?;
        chain_state.flush()?;
        Ok(TransactionInfo::new(
            txn_hash,
            state_root,
            //TODO genesis event.
            HashValue::zero(),
            events,
            gas_used,
            status.vm_status().major_status,
        ))
    }

    pub fn state(&self) -> &ChangeSet {
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
        Ok(Some(genesis))
    }

    pub fn execute(self, storage: Arc<dyn Store>) -> Result<StartupInfo> {
        let Genesis { state, block } = self;

        let chain_state_db = ChainStateDB::new(storage.clone().into_super_arc(), None);
        let transaction_info = Self::execute_genesis_txn(state, &chain_state_db)?;

        ensure!(
            block.header().state_root() == transaction_info.state_root_hash(),
            "Genesis block state root mismatch."
        );

        let txn_accumulator = MerkleAccumulator::new(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            0,
            AccumulatorStoreType::Transaction,
            storage.clone().into_super_arc(),
        )?;
        let txn_info_hash = transaction_info.crypto_hash();

        let (_, _) = txn_accumulator.append(vec![txn_info_hash].as_slice())?;
        txn_accumulator.flush()?;
        let txn_accumulator_info: AccumulatorInfo = (&txn_accumulator).try_into()?;
        ensure!(
            block.header().number() == 0,
            "Genesis block number must is 0."
        );
        debug!("Genesis block id : {:?}", block.header().id());

        ensure!(
            block.header().accumulator_root() == *txn_accumulator_info.get_accumulator_root(),
            "Genesis block accumulator root mismatch."
        );
        //TODO verify consensus header
        storage.commit_block(block.clone(), BlockState::Executed)?;

        let startup_info = StartupInfo::new(block.header().id(), vec![]);
        let block_info = BlockInfo::new_with_accumulator_info(
            block.header().id(),
            txn_accumulator_info,
            Self::genesis_block_accumulator_info(block.header().id(), storage.clone())?,
            U512::zero(),
        );
        debug!("Genesis block_info: {:?}", block_info);
        storage.save_block_info(block_info)?;
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

    fn genesis_block_accumulator_info(
        genesis_block_id: HashValue,
        storage: Arc<dyn Store>,
    ) -> Result<AccumulatorInfo> {
        let accumulator = MerkleAccumulator::new(
            *ACCUMULATOR_PLACEHOLDER_HASH,
            vec![],
            0,
            0,
            AccumulatorStoreType::Block,
            storage.clone().into_super_arc(),
        )?;

        let (_, _) = accumulator.append(vec![genesis_block_id].as_slice())?;
        accumulator.flush()?;
        (&accumulator).try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_state_api::AccountStateReader;
    use starcoin_storage::block_info::BlockInfoStore;
    use starcoin_storage::cache_storage::CacheStorage;
    use starcoin_storage::storage::StorageInstance;
    use starcoin_storage::{BlockStore, IntoSuper, Storage};
    use starcoin_vm_types::account_config::association_address;
    use starcoin_vm_types::on_chain_config::{RegisteredCurrencies, VMConfig, Version};

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
        debug!("build genesis {} for {:?}", genesis, net);
        genesis.save(temp_dir.as_ref())?;
        assert!(!genesis.state.write_set().is_empty());
        let genesis2 = Genesis::load(temp_dir.as_ref())?;
        assert!(genesis2.is_some(), "load genesis fail.");
        let genesis2 = genesis2.unwrap();
        assert_eq!(genesis, genesis2, "genesis save and load different.");

        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance(
            CacheStorage::new(),
        ))?);
        let startup_info = genesis.execute(storage.clone())?;

        let storage2 = Arc::new(Storage::new(StorageInstance::new_cache_instance(
            CacheStorage::new(),
        ))?);
        let startup_info2 = genesis2.execute(storage2)?;

        assert_eq!(
            startup_info, startup_info2,
            "genesis execute startup info different."
        );
        let genesis_block = storage
            .get_block(startup_info.master)?
            .expect("Genesis block must exist.");
        let state_db = ChainStateDB::new(
            storage.clone().into_super_arc(),
            Some(genesis_block.header().state_root()),
        );
        let account_state_reader = AccountStateReader::new(&state_db);
        let account_resource = account_state_reader.get_account_resource(&association_address())?;
        assert!(
            account_resource.is_some(),
            "association account must exist in genesis state."
        );

        let currencies = account_state_reader.get_on_chain_config::<RegisteredCurrencies>();
        assert!(
            currencies.is_some(),
            "RegisteredCurrencies on_chain_config should exist."
        );
        assert!(
            !currencies.unwrap().currency_codes().is_empty(),
            "RegisteredCurrencies should not empty."
        );

        let vm_config = account_state_reader.get_on_chain_config::<VMConfig>();
        assert!(
            vm_config.is_some(),
            "VMConfig on_chain_config should exist."
        );

        let version = account_state_reader.get_on_chain_config::<Version>();
        assert!(version.is_some(), "Version on_chain_config should exist.");

        let block_info = storage
            .get_block_info(genesis_block.header().id())?
            .expect("Genesis block info must exist.");

        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new(
            *txn_accumulator_info.get_accumulator_root(),
            txn_accumulator_info.get_frozen_subtree_roots().clone(),
            txn_accumulator_info.get_num_leaves(),
            txn_accumulator_info.get_num_nodes(),
            AccumulatorStoreType::Transaction,
            storage.clone().into_super_arc(),
        )?;
        //ensure block_accumulator can work.
        txn_accumulator.append(&[HashValue::random()])?;
        txn_accumulator.flush()?;

        let block_accumulator_info = block_info.get_block_accumulator_info();
        let block_accumulator = MerkleAccumulator::new(
            *block_accumulator_info.get_accumulator_root(),
            block_accumulator_info.get_frozen_subtree_roots().clone(),
            block_accumulator_info.get_num_leaves(),
            block_accumulator_info.get_num_nodes(),
            AccumulatorStoreType::Block,
            storage.into_super_arc(),
        )?;
        let hash = block_accumulator.get_leaf(0)?.expect("leaf 0 must exist.");
        assert_eq!(hash, block_info.block_id);
        //ensure block_accumulator can work.
        block_accumulator.append(&[HashValue::random()])?;
        block_accumulator.flush()?;

        Ok(())
    }
}
