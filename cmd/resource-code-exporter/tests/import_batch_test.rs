// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use resource_code_exporter::import_batch::{import_from_statedb_batch, import_from_statedb_adaptive, debug_apply_batch_accounts, debug_commit_phase_missing_node};
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::info;
use starcoin_statedb::{ChainStateDB, ChainStateReader};
use starcoin_transaction_builder::{
    encode_transfer_script_function, peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME,
    DEFAULT_MAX_GAS_AMOUNT,
};
use starcoin_types::{account_address::AccountAddress, vm_error::KeptVMStatus};
use starcoin_vm_types::{
    account_config::{association_address, core_code_address, genesis_address},
    language_storage::{ModuleId, TypeTag},
    on_chain_config,
    state_view::StateReaderExt,
    token::token_code::TokenCode,
    transaction::{Package, Script, ScriptFunction, Transaction, TransactionPayload},
};

use bcs_ext;
use starcoin_crypto::HashValue;
use starcoin_types::account::Account;
use tempfile::TempDir;
use test_helper::executor::{
    association_execute_should_success, compile_modules_with_address, compile_script,
    execute_and_apply, prepare_genesis,
};
use test_helper::txn::create_account_txn_sent_as_association;

use starcoin_chain::verifier::FullVerifier;
use starcoin_config::upgrade_config::vm1_offline_height;
use starcoin_logger::{init_with_default_level, LogPattern};
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::StructTag;
use starcoin_types::multi_transaction::MultiSignedUserTransaction;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::account_config::{AccountResource, BalanceResource};
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::on_chain_config::Version;

pub fn vm1_testnet() -> anyhow::Result<ChainNetwork> {
    let chain_name = "vm1-testnet".to_string();
    let net = ChainNetwork::new_custom(
        chain_name,
        124.into(),
        BuiltinNetworkID::Test.genesis_config().clone(),
        BuiltinNetworkID::Test.genesis_config2().clone(),
    )
    .unwrap();

    let vm1_offline_height = vm1_offline_height(124.into());
    assert_eq!(vm1_offline_height, u64::MAX);

    Ok(net)
}

fn gen_chain_for_test_and_return_statedb(
    net: &ChainNetwork,
) -> anyhow::Result<(BlockChain, ChainStateDB)> {
    let (storage, storage2, chain_info, _) =
        Genesis::init_storage_for_test(net).expect("init storage by genesis fail.");

    let block_chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage.clone(),
        storage2.clone(),
        None,
    )?;
    let state_root = block_chain.chain_state_reader().state_root();
    Ok((block_chain, ChainStateDB::new(storage, Some(state_root))))
}

#[stest::test]
pub fn test_batch_import_from_mainnet_data() -> anyhow::Result<()> {
    init_with_default_level("info", Some(LogPattern::WithLine));

    // 1. vm_testnet
    let net = vm1_testnet()?;

    // 2. Build genesis block into db
    let (_, statedb) = gen_chain_for_test_and_return_statedb(&net)?;

    // 3. unzip from ./test-data/24674819.tar.gz
    let temp_dir = TempDir::new()?;
    let tar_gz_path = std::path::Path::new("./test-data/24674819.tar.gz");

    info!("Extracting tar.gz file from: {}", tar_gz_path.display());

    // Extract the tar.gz file
    let tar_gz_file = std::fs::File::open(tar_gz_path)?;
    let tar_file = flate2::read::GzDecoder::new(tar_gz_file);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    info!(
        "Successfully extracted tar.gz file to: {}",
        temp_dir.path().display()
    );

    // Import the BCS files using batch processing
    let bcs_files = ["24674819.bcs", "24674818.bcs"];
    for bcs_file in &bcs_files {
        let bcs_path = temp_dir.path().join(bcs_file);
        if bcs_path.exists() {
            info!("Importing BCS file: {}", bcs_path.display());
            // Use batch processing with 1000 accounts per batch
            import_from_statedb_batch(&statedb, &bcs_path, None, 1000)?;
            info!("Successfully imported: {}", bcs_file);
        } else {
            info!("BCS file not found: {}", bcs_path.display());
        }
    }

    // 4. Check 0x1 version
    let version = statedb
        .get_on_chain_config::<Version>()?
        .unwrap_or(Version { major: 0 });
    assert_eq!(version.major, 12);

    Ok(())
}

#[stest::test]
pub fn test_adaptive_batch_import_from_mainnet_data() -> anyhow::Result<()> {
    init_with_default_level("info", Some(LogPattern::WithLine));

    // 1. vm_testnet
    let net = vm1_testnet()?;

    // 2. Build genesis block into db
    let (_, statedb) = gen_chain_for_test_and_return_statedb(&net)?;

    // 3. unzip from ./test-data/24674819.tar.gz
    let temp_dir = TempDir::new()?;
    let tar_gz_path = std::path::Path::new("./test-data/24674819.tar.gz");

    info!("Extracting tar.gz file from: {}", tar_gz_path.display());

    // Extract the tar.gz file
    let tar_gz_file = std::fs::File::open(tar_gz_path)?;
    let tar_file = flate2::read::GzDecoder::new(tar_gz_file);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    info!(
        "Successfully extracted tar.gz file to: {}",
        temp_dir.path().display()
    );

    // Import the BCS files using adaptive batch processing
    let bcs_files = ["24674819.bcs", "24674818.bcs"];
    for bcs_file in &bcs_files {
        let bcs_path = temp_dir.path().join(bcs_file);
        if bcs_path.exists() {
            info!("Importing BCS file: {}", bcs_path.display());
            // Use adaptive batch processing
            import_from_statedb_adaptive(&statedb, &bcs_path, None)?;
            info!("Successfully imported: {}", bcs_file);
        } else {
            info!("BCS file not found: {}", bcs_path.display());
        }
    }

    // 4. Check 0x1 version
    let version = statedb
        .get_on_chain_config::<Version>()?
        .unwrap_or(Version { major: 0 });
    assert_eq!(version.major, 12);

    Ok(())
}

#[stest::test]
pub fn test_batch_import_with_small_batch_size() -> anyhow::Result<()> {
    init_with_default_level("info", Some(LogPattern::WithLine));

    // 1. vm_testnet
    let net = vm1_testnet()?;

    // 2. Build genesis block into db
    let (_, statedb) = gen_chain_for_test_and_return_statedb(&net)?;

    // 3. unzip from ./test-data/24674819.tar.gz
    let temp_dir = TempDir::new()?;
    let tar_gz_path = std::path::Path::new("./test-data/24674819.tar.gz");

    info!("Extracting tar.gz file from: {}", tar_gz_path.display());

    // Extract the tar.gz file
    let tar_gz_file = std::fs::File::open(tar_gz_path)?;
    let tar_file = flate2::read::GzDecoder::new(tar_gz_file);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    info!(
        "Successfully extracted tar.gz file to: {}",
        temp_dir.path().display()
    );

    // Import the BCS files using very small batch size for testing
    let bcs_files = ["24674819.bcs", "24674818.bcs"];
    for bcs_file in &bcs_files {
        let bcs_path = temp_dir.path().join(bcs_file);
        if bcs_path.exists() {
            info!("Importing BCS file: {}", bcs_path.display());
            // Use very small batch size (100 accounts per batch) for testing
            import_from_statedb_batch(&statedb, &bcs_path, None, 100)?;
            info!("Successfully imported: {}", bcs_file);
        } else {
            info!("BCS file not found: {}", bcs_path.display());
        }
    }

    // 4. Check 0x1 version
    let version = statedb
        .get_on_chain_config::<Version>()?
        .unwrap_or(Version { major: 0 });
    assert_eq!(version.major, 12);

    Ok(())
}

#[stest::test]
pub fn test_debug_batch_60_missing_node() -> anyhow::Result<()> {
    init_with_default_level("info", Some(LogPattern::WithLine));

    // 1. vm_testnet
    let net = vm1_testnet()?;

    // 2. Build genesis block into db
    let (_, statedb) = gen_chain_for_test_and_return_statedb(&net)?;

    // 3. unzip from ./test-data/24674819.tar.gz
    let temp_dir = TempDir::new()?;
    let tar_gz_path = std::path::Path::new("./test-data/24674819.tar.gz");

    info!("Extracting tar.gz file from: {}", tar_gz_path.display());

    // Extract the tar.gz file
    let tar_gz_file = std::fs::File::open(tar_gz_path)?;
    let tar_file = flate2::read::GzDecoder::new(tar_gz_file);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    info!(
        "Successfully extracted tar.gz file to: {}",
        temp_dir.path().display()
    );

    // 4. Debug batch 60 specifically
    let bcs_files = ["24674819.bcs", "24674818.bcs"];
    for bcs_file in &bcs_files {
        let bcs_path = temp_dir.path().join(bcs_file);
        if bcs_path.exists() {
            info!("Debugging batch 60 from BCS file: {}", bcs_path.display());
            
            // Debug batch 60 (batch_index = 59, 0-based)
            match debug_apply_batch_accounts(&statedb, &bcs_path, 1000, 59) {
                Ok(_) => {
                    info!("Successfully debugged batch 60 from: {}", bcs_file);
                }
                Err(e) => {
                    info!("Error debugging batch 60 from {}: {}", bcs_file, e);
                }
            }
        } else {
            info!("BCS file not found: {}", bcs_path.display());
        }
    }

    Ok(())
}

#[test]
pub fn test_debug_all_batches_for_missing_nodes() -> anyhow::Result<()> {
    init_with_default_level("info", Some(LogPattern::WithLine));

    // 1. vm_testnet
    let net = vm1_testnet()?;

    // 2. Build genesis block into db
    let (_, statedb) = gen_chain_for_test_and_return_statedb(&net)?;

    // 3. unzip from ./test-data/24674819.tar.gz
    let temp_dir = TempDir::new()?;
    let tar_gz_path = std::path::Path::new("./test-data/24674819.tar.gz");

    info!("Extracting tar.gz file from: {}", tar_gz_path.display());

    // Extract the tar.gz file
    let tar_gz_file = std::fs::File::open(tar_gz_path)?;
    let tar_file = flate2::read::GzDecoder::new(tar_gz_file);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    info!(
        "Successfully extracted tar.gz file to: {}",
        temp_dir.path().display()
    );

    // 4. Debug all batches to find missing nodes
    let bcs_files = [
        "24674819.bcs",
        // "24674818.bcs"
    ];
    for bcs_file in &bcs_files {
        let bcs_path = temp_dir.path().join(bcs_file);
        if bcs_path.exists() {
            info!("Debugging all batches from BCS file: {}", bcs_path.display());
            
            // Read BCS file to get total number of accounts
            let bcs_data = std::fs::read(&bcs_path)?;
            let chain_state_set: ChainStateSet = bcs_ext::from_bytes(&bcs_data)?;
            let total_accounts = chain_state_set.len();
            let batch_size = 10000;
            let total_batches = (total_accounts + batch_size - 1) / batch_size;
            
            info!("Total accounts: {}, Total batches: {}", total_accounts, total_batches);
            
            // Debug each batch
            for batch_idx in 0..total_batches {
                info!("Debugging batch {}/{}", batch_idx + 1, total_batches);
                match debug_apply_batch_accounts(&statedb, &bcs_path, batch_size, batch_idx) {
                    Ok(_) => {
                        info!("Successfully debugged batch {}/{}", batch_idx + 1, total_batches);
                    }
                    Err(e) => {
                        info!("Error debugging batch {}/{}: {}", batch_idx + 1, total_batches, e);
                    }
                }
            }
        } else {
            info!("BCS file not found: {}", bcs_path.display());
        }
    }

    Ok(())
}

#[stest::test]
pub fn test_debug_commit_phase_missing_node() -> anyhow::Result<()> {
    init_with_default_level("info", Some(LogPattern::WithLine));

    // 1. vm_testnet
    let net = vm1_testnet()?;

    // 2. Build genesis block into db
    let (_, statedb) = gen_chain_for_test_and_return_statedb(&net)?;

    // 3. unzip from ./test-data/24674819.tar.gz
    let temp_dir = TempDir::new()?;
    let tar_gz_path = std::path::Path::new("./test-data/24674819.tar.gz");

    info!("Extracting tar.gz file from: {}", tar_gz_path.display());

    // Extract the tar.gz file
    let tar_gz_file = std::fs::File::open(tar_gz_path)?;
    let tar_file = flate2::read::GzDecoder::new(tar_gz_file);
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)?;

    info!(
        "Successfully extracted tar.gz file to: {}",
        temp_dir.path().display()
    );

    // 4. Debug commit phase for batch 60 specifically
    let bcs_files = ["24674819.bcs", "24674818.bcs"];
    for bcs_file in &bcs_files {
        let bcs_path = temp_dir.path().join(bcs_file);
        if bcs_path.exists() {
            info!("Debugging commit phase for batch 60 from BCS file: {}", bcs_path.display());
            
            // Debug commit phase for batch 60 (batch_index = 59, 0-based)
            match debug_commit_phase_missing_node(&statedb, &bcs_path, 1000, 59) {
                Ok(_) => {
                    info!("Successfully debugged commit phase for batch 60 from: {}", bcs_file);
                }
                Err(e) => {
                    info!("Error debugging commit phase for batch 60 from {}: {}", bcs_file, e);
                }
            }
        } else {
            info!("BCS file not found: {}", bcs_path.display());
        }
    }

    Ok(())
} 