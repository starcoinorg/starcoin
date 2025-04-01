use anyhow::Result;

use starcoin_types2::account_config::CORE_CODE_ADDRESS;
use starcoin_vm2_crypto::ed25519::genesis_key_pair;
use starcoin_vm2_executor::executor2::execute_block_transactions_with_chain_id;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_transaction_builder::{build_init_script, build_stdlib_package_for_test};

use starcoin_vm2_types::{
    genesis_config::ChainNetwork,
    transaction::{
        Package, RawUserTransaction, SignedUserTransaction, Transaction, TransactionPayload,
    },
};

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
    let package = build_stdlib_package_for_test(0, Some(entry_func))?;
    let genesis_txn = test_build_genesis_transaction_with_package(&net, package)?;

    // Execute with vm 2
    // let storage = Arc::new(Storage2::new(StorageInstance2::new_cache_instance())?);
    let statedb = ChainStateDB2::mock();
    let txn_outputs = execute_block_transactions_with_chain_id(
        &statedb,
        vec![Transaction::UserTransaction(genesis_txn)],
        0,
        None,
        &net.chain_id(),
    )?;
    assert!(txn_outputs.len() > 0);

    Ok(())
}
