// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Result};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::BlockChain;
use starcoin_config::{genesis_key_pair, ChainNetwork};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_state_api::ChainState;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_transaction_builder::{build_stdlib_package, StdLibOptions};
use starcoin_types::startup_info::StartupInfo;
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
use std::path::Path;
use std::sync::Arc;
use traits::{ChainReader, ChainWriter};

mod errors;

pub use errors::GenesisError;
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_vm_types::genesis_config::{BuiltinNetworkID, ChainNetworkID};

pub static GENESIS_GENERATED_DIR: &str = "generated";

const HALLEY_GENESIS_BYTES: &[u8] = std::include_bytes!("../generated/halley/genesis");
const PROXIMA_GENESIS_BYTES: &[u8] = std::include_bytes!("../generated/proxima/genesis");
const MAIN_GENESIS_BYTES: &[u8] = std::include_bytes!("../generated/main/genesis");

static FRESH_TEST_GENESIS: Lazy<Genesis> = Lazy::new(|| {
    Genesis::build(&ChainNetwork::new_builtin(BuiltinNetworkID::Test))
        .unwrap_or_else(|e| panic!("build genesis for {} fail: {:?}", BuiltinNetworkID::Test, e))
});

static FRESH_DEV_GENESIS: Lazy<Genesis> = Lazy::new(|| {
    Genesis::build(&ChainNetwork::new_builtin(BuiltinNetworkID::Dev))
        .unwrap_or_else(|e| panic!("build genesis for {} fail: {:?}", BuiltinNetworkID::Dev, e))
});

static FRESH_HALLEY_GENESIS: Lazy<Genesis> = Lazy::new(|| {
    Genesis::build(&ChainNetwork::new_builtin(BuiltinNetworkID::Halley)).unwrap_or_else(|e| {
        panic!(
            "build genesis for {} fail: {:?}",
            BuiltinNetworkID::Halley,
            e
        )
    })
});

static FRESH_PROXIMA_GENESIS: Lazy<Genesis> = Lazy::new(|| {
    Genesis::build(&ChainNetwork::new_builtin(BuiltinNetworkID::Proxima)).unwrap_or_else(|e| {
        panic!(
            "build genesis for {} fail: {:?}",
            BuiltinNetworkID::Proxima,
            e
        )
    })
});

static FRESH_MAIN_GENESIS: Lazy<Genesis> = Lazy::new(|| {
    Genesis::build(&ChainNetwork::new_builtin(BuiltinNetworkID::Main))
        .unwrap_or_else(|e| panic!("build genesis for {} fail: {:?}", BuiltinNetworkID::Main, e))
});

pub enum GenesisOpt {
    /// Load generated genesis
    Generated,
    /// Regenerate genesis
    Fresh,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Genesis {
    block: Block,
}

impl Display for Genesis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Genesis {{")?;
        write!(f, "block: {:?}", self.block.header.id())?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl Genesis {
    pub const GENESIS_FILE_NAME: &'static str = "genesis";

    pub fn load_by_opt(option: GenesisOpt, net: &ChainNetwork) -> Result<Self> {
        match (option, net.id()) {
            (GenesisOpt::Generated, ChainNetworkID::Builtin(net)) => Self::load_generated(*net),
            (GenesisOpt::Fresh, ChainNetworkID::Builtin(net)) => match net {
                BuiltinNetworkID::Test => Ok(FRESH_TEST_GENESIS.clone()),
                BuiltinNetworkID::Dev => Ok(FRESH_DEV_GENESIS.clone()),
                BuiltinNetworkID::Halley => Ok(FRESH_HALLEY_GENESIS.clone()),
                BuiltinNetworkID::Proxima => Ok(FRESH_PROXIMA_GENESIS.clone()),
                BuiltinNetworkID::Main => Ok(FRESH_MAIN_GENESIS.clone()),
            },
            (_, _) => Self::build(net),
        }
    }

    /// Load pre generated genesis.
    pub fn load(net: &ChainNetwork) -> Result<Self> {
        // test and dev always use Fresh genesis.
        if net.is_test() || net.is_dev() {
            Self::load_by_opt(GenesisOpt::Fresh, net)
        } else {
            Self::load_by_opt(GenesisOpt::Generated, net)
        }
    }

    /// Build fresh genesis
    pub(crate) fn build(net: &ChainNetwork) -> Result<Self> {
        debug!("Init genesis for {}", net);
        let block = Self::build_genesis_block(net)?;
        assert_eq!(block.header().number(), 0);
        debug!("Genesis block id : {:?}", block.header().id());
        let genesis = Self { block };
        Ok(genesis)
    }

    fn build_genesis_block(net: &ChainNetwork) -> Result<Block> {
        let genesis_config = net.genesis_config();

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
            genesis_config.parent_hash,
            genesis_config.timestamp,
            accumulator_root,
            transaction_info.state_root_hash(),
            genesis_config.difficulty,
            genesis_config.nonce,
            txn,
        ))
    }

    pub fn build_genesis_transaction(net: &ChainNetwork) -> Result<SignedUserTransaction> {
        let package = build_stdlib_package(
            net,
            if net.is_test() {
                StdLibOptions::Fresh
            } else {
                StdLibOptions::Compiled(net.stdlib_version())
            },
            true,
        )?;
        let txn = RawUserTransaction::new(
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

        let output = starcoin_executor::execute_transactions(chain_state.as_super(), vec![txn])?
            .pop()
            .expect("Execute output must exist.");
        let (write_set, events, gas_used, _, status) = output.into_inner();
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
        let genesis = scs::from_bytes(&content)?;
        Ok(Some(genesis))
    }

    fn genesis_bytes(net: BuiltinNetworkID) -> &'static [u8] {
        match net {
            BuiltinNetworkID::Test => unreachable!(),
            BuiltinNetworkID::Dev => unreachable!(),
            BuiltinNetworkID::Halley => HALLEY_GENESIS_BYTES,
            BuiltinNetworkID::Proxima => PROXIMA_GENESIS_BYTES,
            BuiltinNetworkID::Main => MAIN_GENESIS_BYTES,
        }
    }

    pub fn load_generated(net: BuiltinNetworkID) -> Result<Self> {
        let bytes = Self::genesis_bytes(net);
        scs::from_bytes(bytes)
    }

    pub fn execute_genesis_block(
        &self,
        net: &ChainNetwork,
        storage: Arc<dyn Store>,
    ) -> Result<StartupInfo> {
        let mut genesis_chain = BlockChain::init_empty_chain(net.time_service(), storage.clone());
        genesis_chain.apply(self.block.clone())?;
        let startup_info = StartupInfo::new(genesis_chain.current_header().id());
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
        let genesis_file = data_dir.join(Self::GENESIS_FILE_NAME);
        let mut file = File::create(genesis_file)?;
        let contents = scs::to_bytes(self)?;
        file.write_all(&contents)?;
        Ok(())
    }

    fn load_and_check_genesis(net: &ChainNetwork, data_dir: &Path, init: bool) -> Result<Genesis> {
        let genesis = match Genesis::load_from_dir(data_dir) {
            Ok(Some(genesis)) => {
                let expect_genesis = Genesis::load(net)?;
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
                    let genesis = Genesis::load(net)?;
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
    ) -> Result<(StartupInfo, Genesis)> {
        debug!("load startup_info.");
        let (startup_info, genesis) = match storage.get_startup_info() {
            Ok(Some(startup_info)) => {
                info!("Get startup info from db");
                info!("Check genesis file.");
                let genesis = Self::load_and_check_genesis(net, data_dir, false)?;
                match storage.get_block(genesis.block().header().id()) {
                    Ok(Some(block)) => {
                        if *genesis.block() == block {
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
                        return Err(GenesisError::GenesisNotExist("database".to_owned()).into())
                    }
                    Err(e) => return Err(GenesisError::GenesisLoadFailure(e).into()),
                }
                (startup_info, genesis)
            }
            Ok(None) => {
                let genesis = Self::load_and_check_genesis(net, data_dir, true)?;
                let startup_info = genesis.execute_genesis_block(net, storage.clone())?;
                (startup_info, genesis)
            }
            Err(e) => return Err(GenesisError::GenesisLoadFailure(e).into()),
        };
        //TODO add init time for TimeService
        Ok((startup_info, genesis))
    }

    pub fn init_storage_for_test(
        net: &ChainNetwork,
    ) -> Result<(Arc<Storage>, StartupInfo, HashValue)> {
        debug!("init storage by genesis for test.");
        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
        let genesis = Genesis::load(net)?;
        let genesis_hash = genesis.block.header().id();
        let startup_info = genesis.execute_genesis_block(net, storage.clone())?;
        Ok((storage, startup_info, genesis_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_crypto::HashValue;
    use starcoin_state_api::AccountStateReader;
    use starcoin_storage::block_info::BlockInfoStore;
    use starcoin_storage::storage::StorageInstance;
    use starcoin_storage::{BlockStore, IntoSuper, Storage};
    use starcoin_types::account_config::genesis_address;
    use starcoin_vm_types::account_config::association_address;
    use starcoin_vm_types::genesis_config::ChainId;
    use starcoin_vm_types::on_chain_config::DaoConfig;
    use starcoin_vm_types::on_chain_config::{ConsensusConfig, VMConfig, Version};
    use starcoin_vm_types::on_chain_resource::Epoch;

    #[stest::test]
    pub fn test_genesis_load() -> Result<()> {
        for id in BuiltinNetworkID::networks() {
            let net = ChainNetwork::new_builtin(id);
            Genesis::load(&net)?;
        }
        Ok(())
    }

    #[stest::test(timeout = 120)]
    pub fn test_builtin_genesis() -> Result<()> {
        for id in BuiltinNetworkID::networks() {
            let net = ChainNetwork::new_builtin(id);
            let temp_dir = starcoin_config::temp_path();
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
        let temp_dir = starcoin_config::temp_path();
        do_test_genesis(&net, temp_dir.path())
    }

    pub fn do_test_genesis(net: &ChainNetwork, data_dir: &Path) -> Result<()> {
        let storage1 = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
        let (startup_info1, genesis1) = Genesis::init_and_check_storage(net, storage1, data_dir)?;

        let storage2 = Arc::new(Storage::new(StorageInstance::new_cache_instance())?);
        let (startup_info2, genesis2) =
            Genesis::init_and_check_storage(net, storage2.clone(), data_dir)?;

        assert_eq!(
            genesis1, genesis2,
            "genesis execute startup info different."
        );

        assert_eq!(
            startup_info1, startup_info2,
            "genesis execute startup info different."
        );

        let genesis_block = storage2
            .get_block(startup_info2.master)?
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

        let vm_config = account_state_reader.get_on_chain_config::<VMConfig>()?;
        assert!(
            vm_config.is_some(),
            "VMConfig on_chain_config should exist."
        );

        let consensus_config = account_state_reader.get_on_chain_config::<ConsensusConfig>()?;
        assert!(
            consensus_config.is_some(),
            "ConsensusConfig on_chain_config should exist."
        );

        let dao_config = account_state_reader.get_on_chain_config::<DaoConfig>()?;
        assert!(
            dao_config.is_some(),
            "DaoConfig on_chain_config should exist."
        );

        let version = account_state_reader.get_on_chain_config::<Version>()?;
        assert!(version.is_some(), "Version on_chain_config should exist.");

        let block_info = storage2
            .get_block_info(genesis_block.header().id())?
            .expect("Genesis block info must exist.");

        let txn_accumulator_info = block_info.get_txn_accumulator_info();
        let txn_accumulator = MerkleAccumulator::new_with_info(
            txn_accumulator_info.clone(),
            storage2.get_accumulator_store(AccumulatorStoreType::Transaction),
        );
        //ensure block_accumulator can work.
        txn_accumulator.append(&[HashValue::random()])?;
        txn_accumulator.flush()?;

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
