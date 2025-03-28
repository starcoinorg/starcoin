use anyhow::Result;
use std::sync::Arc;

//use starcoin_executor::do_execute_block_transactions;
use starcoin_executor::executor2::execute_block_transactions;
use starcoin_types2::account_config::CORE_CODE_ADDRESS;
use starcoin_vm2_crypto::ed25519::genesis_key_pair;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_storage::{storage::StorageInstance as StorageInstance2, Storage as Storage2};
use starcoin_vm2_transaction_builder::{build_init_script, build_stdlib_package_for_test};

use starcoin_vm2_types::{
    genesis_config::ChainNetwork,
    transaction::{Package, RawUserTransaction, SignedUserTransaction, TransactionPayload},
};
use starcoin_transaction_builder::build_signed_empty_txn;

fn test_build_genesis_transaction_with_package(
    net: &ChainNetwork,
    package: Package,
) -> Result<SignedUserTransaction> {
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

#[stest::test]
pub fn test_build_framework_2_genesis() -> Result<()> {
    // Read all packages from testnet.mrb
    let net = ChainNetwork::new_test();

    let entry_func = build_init_script(&net);
    let package = build_stdlib_package_for_test(1, Some(entry_func))?;
    let signed_txn = test_build_genesis_transaction_with_package(&net, package)?;


    // Execute with vm 2
    let storage = Arc::new(Storage2::new(StorageInstance2::new_cache_instance())?);
    let statedb = ChainStateDB2::new(storage.clone(), None);
    let txn_outputs = execute_block_transactions(&statedb, vec![signed_txn], 0, None)?;
    assert!(txn_outputs.len() > 0);

    Ok(())
}
