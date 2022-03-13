// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, format_err, Result};
use include_dir::include_dir;
use include_dir::Dir;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::{
    genesis_key_pair, BuiltinNetworkID, ChainNetwork, ChainNetworkID, GenesisBlockParameter,
};
use starcoin_logger::prelude::*;
use starcoin_state_api::ChainState;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_transaction_builder::{build_stdlib_package, StdLibOptions};
use starcoin_types::startup_info::{ChainInfo, StartupInfo};
use starcoin_types::transaction::TransactionInfo;
use starcoin_types::{block::Block, transaction::Transaction};
use starcoin_vm_types::account_config::CORE_CODE_ADDRESS;
use starcoin_vm_types::transaction::{
    RawUserTransaction, SignedUserTransaction, TransactionPayload,
};
use starcoin_vm_types::vm_status::KeptVMStatus;
use std::fmt::Display;
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod errors;
pub use errors::GenesisError;

pub static GENESIS_GENERATED_DIR: &str = "generated";
pub const GENESIS_DIR: Dir = include_dir!("generated");

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Genesis {
    block: Block,
}

impl Display for Genesis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Genesis {{")?;
        write!(f, "block: {:?}; ", self.block.header.id())?;
        write!(f, "parent_hash: {:?}; ", self.block.header.parent_hash())?;
        write!(f, "timestamp: {:?}; ", self.block.header.timestamp())?;
        write!(
            f,
            "accumulator_root: {:?}; ",
            self.block.header.txn_accumulator_root()
        )?;
        write!(f, "state_root: {:?}; ", self.block.header.state_root())?;
        write!(f, "difficulty: {:?}; ", self.block.header.difficulty())?;
        write!(f, "body_hash: {:?}; ", self.block.header.body_hash())?;
        write!(f, "chain_id: {:?}; ", self.block.header.chain_id())?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl Genesis {
    pub const GENESIS_FILE_NAME: &'static str = "genesis";

    /// Load Load pre generated genesis, only support builtin network.
    pub fn load(net: &ChainNetwork) -> Result<Option<Self>> {
        match net.id() {
            ChainNetworkID::Builtin(id) => Self::load_generated(*id),
            _ => Ok(None),
        }
    }

    /// Load pre generated genesis.
    pub fn load_or_build(net: &ChainNetwork) -> Result<Self> {
        // test and dev always use Fresh genesis.
        if net.is_test() || net.is_dev() {
            Self::build(net)
        } else {
            match Self::load(net)? {
                Some(genesis) => Ok(genesis),
                None => Self::build(net),
            }
        }
    }

    /// Build fresh genesis
    pub fn build(net: &ChainNetwork) -> Result<Self> {
        debug!("Init genesis for {}", net);
        let block = Self::build_genesis_block(net)?;
        assert_eq!(block.header().number(), 0);
        debug!("Genesis block id : {:?}", block.header().id());
        let genesis = Self { block };
        Ok(genesis)
    }

    fn build_genesis_block(net: &ChainNetwork) -> Result<Block> {
        let genesis_config = net.genesis_config();
        if let Some(GenesisBlockParameter {
            parent_hash,
            timestamp,
            difficulty,
        }) = genesis_config.genesis_block_parameter()
        {
            let txn = Self::build_genesis_transaction(net)?;

            let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
            let chain_state_db = ChainStateDB::new(storage.clone(), None);

            let transaction_info = Self::execute_genesis_txn(&chain_state_db, txn.clone())?;

            let accumulator = MerkleAccumulator::new_with_info(
                AccumulatorInfo::default(),
                storage.get_accumulator_store(AccumulatorStoreType::Transaction),
            );
            let txn_info_hash = transaction_info.id();

            let accumulator_root = accumulator.append(vec![txn_info_hash].as_slice())?;
            accumulator.flush()?;
            Ok(Block::genesis_block(
                *parent_hash,
                *timestamp,
                accumulator_root,
                transaction_info.state_root_hash(),
                *difficulty,
                txn,
            ))
        } else {
            bail!("{}'s genesis config not ready to build genesis block", net);
        }
    }

    pub fn build_genesis_transaction(net: &ChainNetwork) -> Result<SignedUserTransaction> {
        let package = build_stdlib_package(
            net,
            if net.is_test() {
                StdLibOptions::Fresh
            } else {
                StdLibOptions::Compiled(net.stdlib_version())
            },
        )?;
        let txn = RawUserTransaction::new_with_default_gas_token(
            CORE_CODE_ADDRESS,
            0,
            TransactionPayload::Package(package),
            0,
            0,
            1, // init to 1 to pass time check
            net.chain_id(),
        );
        let (genesis_private_key, genesis_public_key) = genesis_key_pair();
        let sign_txn = txn.sign(&genesis_private_key, genesis_public_key)?;
        Ok(sign_txn.into_inner())
    }

    pub fn execute_genesis_txn(
        chain_state: &dyn ChainState,
        txn: SignedUserTransaction,
    ) -> Result<TransactionInfo> {
        let txn = Transaction::UserTransaction(txn);
        let txn_hash = txn.id();

        let output =
            starcoin_executor::execute_transactions(chain_state.as_super(), vec![txn], None)?
                .pop()
                .expect("Execute output must exist.");
        let (write_set, events, gas_used, status) = output.into_inner();
        assert_eq!(gas_used, 0, "Genesis txn output's gas_used must be zero");
        let keep_status = status
            .status()
            .map_err(|e| format_err!("Genesis txn is discard by: {:?}", e))?;
        ensure!(
            keep_status == KeptVMStatus::Executed,
            "Genesis txn execute fail for: {:?}",
            keep_status
        );
        chain_state.apply_write_set(write_set)?;
        let state_root = chain_state.commit()?;
        chain_state.flush()?;
        Ok(TransactionInfo::new(
            txn_hash,
            state_root,
            events.as_slice(),
            gas_used,
            keep_status,
        ))
    }

    pub fn block(&self) -> &Block {
        &self.block
    }

    pub fn load_from_dir<P>(data_dir: P) -> Result<Option<Self>>
    where
        P: AsRef<Path>,
    {
        let genesis_file_path = data_dir.as_ref().join(Self::GENESIS_FILE_NAME);
        if !genesis_file_path.exists() {
            return Ok(None);
        }
        let mut genesis_file = File::open(genesis_file_path)?;
        let mut content = vec![];
        genesis_file.read_to_end(&mut content)?;
        let genesis = bcs_ext::from_bytes(&content)?;
        Ok(Some(genesis))
    }

    fn genesis_bytes(net: BuiltinNetworkID) -> Option<&'static [u8]> {
        let path = PathBuf::from(net.to_string()).join("genesis");
        GENESIS_DIR
            .get_file(path.as_path())
            .map(|file| file.contents())
    }

    pub fn load_generated(net: BuiltinNetworkID) -> Result<Option<Self>> {
        match Self::genesis_bytes(net) {
            Some(bytes) => Ok(Some(bcs_ext::from_bytes::<Genesis>(bytes)?)),
            None => Ok(None),
        }
    }

    pub fn execute_genesis_block(
        &self,
        net: &ChainNetwork,
        storage: Arc<dyn Store>,
    ) -> Result<ChainInfo> {
        storage.save_genesis(self.block.id())?;
        let genesis_chain = BlockChain::new_with_genesis(
            net.time_service(),
            storage.clone(),
            net.genesis_epoch(),
            self.block.clone(),
        )?;
        let startup_info = StartupInfo::new(genesis_chain.current_header().id());
        storage.save_startup_info(startup_info)?;
        storage
            .get_chain_info()?
            .ok_or_else(|| format_err!("ChainInfo should exist after genesis block executed."))
    }

    pub fn save<P>(&self, data_dir: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let data_dir = data_dir.as_ref();
        if !data_dir.exists() {
            create_dir_all(data_dir)?;
        }
        let genesis_file = data_dir.join(Self::GENESIS_FILE_NAME);
        let mut file = File::create(genesis_file)?;
        let contents = bcs_ext::to_bytes(self)?;
        file.write_all(&contents)?;
        Ok(())
    }

    fn load_and_check_genesis(net: &ChainNetwork, data_dir: &Path, init: bool) -> Result<Genesis> {
        let genesis = match Genesis::load_from_dir(data_dir) {
            Ok(Some(genesis)) => {
                let expect_genesis = Genesis::load_or_build(net)?;
                if genesis.block().header().id() != expect_genesis.block().header().id() {
                    return Err(GenesisError::GenesisVersionMismatch {
                        expect: expect_genesis.block.header.id(),
                        real: genesis.block.header.id(),
                    }
                    .into());
                }
                genesis
            }
            Err(e) => return Err(GenesisError::GenesisLoadFailure(e).into()),
            Ok(None) => {
                if init {
                    let genesis = Genesis::load_or_build(net)?;
                    genesis.save(data_dir)?;
                    info!("Build and save new genesis: {}", genesis);
                    genesis
                } else {
                    return Err(GenesisError::GenesisNotExist("data_dir".to_owned()).into());
                }
            }
        };
        Ok(genesis)
    }

    pub fn init_and_check_storage(
        net: &ChainNetwork,
        storage: Arc<Storage>,
        data_dir: &Path,
    ) -> Result<(ChainInfo, Genesis)> {
        debug!("load startup_info.");
        let (chain_info, genesis) = match storage.get_chain_info() {
            Ok(Some(chain_info)) => {
                debug!("Get chain info {:?} from db", chain_info);
                info!("Check genesis file.");
                let genesis = Self::load_and_check_genesis(net, data_dir, false)?;
                match storage.get_block(genesis.block().header().id()) {
                    Ok(Some(block)) => {
                        if *genesis.block() == block && chain_info.genesis_hash() == block.id() {
                            info!("Check genesis db block ok!");
                        } else {
                            return Err(GenesisError::GenesisVersionMismatch {
                                expect: genesis.block.header.id(),
                                real: block.header.id(),
                            }
                            .into());
                        }
                    }
                    Ok(None) => {
                        return Err(GenesisError::GenesisNotExist("database".to_owned()).into());
                    }
                    Err(e) => return Err(GenesisError::GenesisLoadFailure(e).into()),
                }
                (chain_info, genesis)
            }
            Ok(None) => {
                let genesis = Self::load_and_check_genesis(net, data_dir, true)?;
                let chain_info = genesis.execute_genesis_block(net, storage.clone())?;
                (chain_info, genesis)
            }
            Err(e) => return Err(GenesisError::GenesisLoadFailure(e).into()),
        };
        //TODO add init time for TimeService
        Ok((chain_info, genesis))
    }

    pub fn init_storage_for_test(net: &ChainNetwork) -> Result<(Arc<Storage>, ChainInfo, Genesis)> {
        debug!("init storage by genesis for test.");
        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
        let genesis = Genesis::load_or_build(net)?;
        let chain_info = genesis.execute_genesis_block(net, storage.clone())?;
        Ok((storage, chain_info, genesis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_crypto::HashValue;
    use starcoin_state_api::AccountStateReader;
    use starcoin_storage::block_info::BlockInfoStore;
    use starcoin_storage::storage::StorageInstance;
    use starcoin_storage::{BlockStore, BlockTransactionInfoStore, IntoSuper, Storage};
    use starcoin_types::account_config::{genesis_address, ModuleUpgradeStrategy};
    use starcoin_vm_types::account_config::association_address;
    use starcoin_vm_types::genesis_config::ChainId;
    use starcoin_vm_types::on_chain_config::{ConsensusConfig, Version};
    use starcoin_vm_types::on_chain_config::{DaoConfig, TransactionPublishOption};
    use starcoin_vm_types::on_chain_resource::Epoch;

    #[stest::test]
    pub fn test_genesis_load() -> Result<()> {
        for id in BuiltinNetworkID::networks() {
            info!("test {} genesis load", id);
            let net = ChainNetwork::new_builtin(id);
            if !net.is_ready() {
                continue;
            }
            Genesis::load_or_build(&net)?;
        }
        Ok(())
    }

    #[stest::test]
    pub fn test_builtin_genesis() -> Result<()> {
        for id in BuiltinNetworkID::networks() {
            if !id.genesis_config().is_ready() {
                continue;
            }
            let net = ChainNetwork::new_builtin(id);
            let temp_dir = starcoin_config::temp_dir();
            do_test_genesis(&net, temp_dir.path())?;
        }
        Ok(())
    }

    #[stest::test]
    pub fn test_custom_genesis() -> Result<()> {
        let net = ChainNetwork::new_custom(
            "testx".to_string(),
            ChainId::new(123),
            BuiltinNetworkID::Test.genesis_config().clone(),
        )?;
        let temp_dir = starcoin_config::temp_dir();
        do_test_genesis(&net, temp_dir.path())
    }

    pub fn do_test_genesis(net: &ChainNetwork, data_dir: &Path) -> Result<()> {
        let storage1 = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
        let (chain_info1, genesis1) =
            Genesis::init_and_check_storage(net, storage1.clone(), data_dir)?;

        let storage2 = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
        let (chain_info2, genesis2) =
            Genesis::init_and_check_storage(net, storage2.clone(), data_dir)?;

        assert_eq!(genesis1, genesis2, "genesis execute chain info different.");

        assert_eq!(
            chain_info1, chain_info2,
            "genesis execute chain info different."
        );

        let genesis_block = storage2
            .get_block(chain_info2.status().head().id())?
            .expect("Genesis block must exist.");

        let state_db = ChainStateDB::new(
            storage2.clone().into_super_arc(),
            Some(genesis_block.header().state_root()),
        );
        let account_state_reader = AccountStateReader::new(&state_db);
        let chain_id = account_state_reader.get_chain_id()?;
        assert_eq!(
            net.chain_id(),
            chain_id,
            "chain id in Move resource should equals ChainNetwork's chain_id."
        );
        let genesis_account_resource =
            account_state_reader.get_account_resource(&genesis_address())?;
        assert!(
            genesis_account_resource.is_some(),
            "genesis account must exist in genesis state."
        );

        let genesis_balance = account_state_reader.get_balance(&genesis_address())?;
        assert!(
            genesis_balance.is_some(),
            "genesis account balance must exist in genesis state."
        );

        let association_account_resource =
            account_state_reader.get_account_resource(&association_address())?;
        assert!(
            association_account_resource.is_some(),
            "association account must exist in genesis state."
        );

        let association_balance = account_state_reader.get_balance(&association_address())?;

        assert!(
            association_balance.is_some(),
            "association account balance must exist in genesis state."
        );

        // let vm_config = account_state_reader.get_on_chain_config::<VMConfig>()?;
        // assert!(
        //     vm_config.is_some(),
        //     "VMConfig on_chain_config should exist."
        // );
        // assert_eq!(vm_config.as_ref().unwrap(), &net.genesis_config().vm_config);

        let vm_publish_option =
            account_state_reader.get_on_chain_config::<TransactionPublishOption>()?;
        assert!(
            vm_publish_option.is_some(),
            "vm_publish_option on_chain_config should exist."
        );
        assert_eq!(
            vm_publish_option.as_ref().unwrap(),
            &net.genesis_config().publishing_option
        );

        let consensus_config = account_state_reader.get_on_chain_config::<ConsensusConfig>()?;
        assert!(
            consensus_config.is_some(),
            "ConsensusConfig on_chain_config should exist."
        );
        assert_eq!(
            consensus_config.as_ref().unwrap(),
            &net.genesis_config().consensus_config
        );

        let dao_config = account_state_reader.get_on_chain_config::<DaoConfig>()?;
        assert!(
            dao_config.is_some(),
            "DaoConfig on_chain_config should exist."
        );

        let version = account_state_reader.get_on_chain_config::<Version>()?;
        assert!(version.is_some(), "Version on_chain_config should exist.");
        assert_eq!(
            version.as_ref().unwrap().major,
            net.genesis_config().stdlib_version.version()
        );

        let module_upgrade_strategy =
            account_state_reader.get_resource::<ModuleUpgradeStrategy>(genesis_address())?;
        assert!(
            module_upgrade_strategy.is_some(),
            "ModuleUpgradeStrategy should exist."
        );
        assert!(
            module_upgrade_strategy.unwrap().two_phase(),
            "ModuleUpgradeStrategy should be STRATEGY_TWO_PHASE."
        );

        let block_info = storage2
            .get_block_info(genesis_block.header().id())?
            .expect("Genesis block info must exist.");

        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        assert_eq!(txn_accumulator_info.num_leaves, 1);
        //assert_eq!(txn_accumulator_info.frozen_subtree_roots.len(), 1);

        let txn_accumulator = MerkleAccumulator::new_with_info(
            txn_accumulator_info.clone(),
            storage2.get_accumulator_store(AccumulatorStoreType::Transaction),
        );

        let genesis_txn = genesis_block.body.transactions.get(0).cloned().unwrap();
        assert_eq!(
            txn_accumulator.get_leaf(0).unwrap().unwrap(),
            storage1
                .get_transaction_info_by_txn_hash(genesis_txn.id())
                .unwrap()
                .pop()
                .unwrap()
                .id(),
            "block metadata txn hash"
        );

        let block_accumulator_info = block_info.get_block_accumulator_info();
        let block_accumulator = MerkleAccumulator::new_with_info(
            block_accumulator_info.clone(),
            storage2.get_accumulator_store(AccumulatorStoreType::Block),
        );
        let hash = block_accumulator.get_leaf(0)?.expect("leaf 0 must exist.");
        assert_eq!(hash, block_info.block_id);
        //ensure block_accumulator can work.
        block_accumulator.append(&[HashValue::random()])?;
        block_accumulator.flush()?;

        let epoch = account_state_reader.get_resource::<Epoch>(genesis_address())?;
        assert!(epoch.is_some(), "Epoch resource should exist.");

        Ok(())
    }
}
