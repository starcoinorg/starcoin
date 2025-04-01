use starcoin_executor::executor2::{
    execute_block_transactions, execute_block_transactions_with_chain_id,
};
use starcoin_vm2_crypto::ed25519::genesis_key_pair;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_transaction_builder::{build_init_script, build_stdlib_package_for_test};
use starcoin_vm2_types::{
    account_config::CORE_CODE_ADDRESS,
    genesis_config::ChainNetwork,
    transaction::{
        Package, RawUserTransaction, SignedUserTransaction, Transaction, TransactionPayload,
    },
};

fn build_genesis_transaction_with_package(
    net: &ChainNetwork,
    package: Package,
) -> anyhow::Result<SignedUserTransaction> {
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

fn build_genesis_transaction(net: &ChainNetwork) -> anyhow::Result<SignedUserTransaction> {
    let entry_func = build_init_script(&net);
    let package = build_stdlib_package_for_test(0, Some(entry_func))?;
    build_genesis_transaction_with_package(&net, package)
}

pub fn prepare_genesis() -> anyhow::Result<(ChainStateDB, ChainNetwork)> {
    let net = ChainNetwork::new_test();
    let chain_state = ChainStateDB::mock();
    let genesis_txn = build_genesis_transaction(&net).unwrap();

    // Execute with vm 2
    // let storage = Arc::new(Storage2::new(StorageInstance2::new_cache_instance())?);
    let statedb = ChainStateDB::mock();
    let txn_outputs = execute_block_transactions_with_chain_id(
        &statedb,
        vec![Transaction::UserTransaction(genesis_txn)],
        0,
        None,
        &net.chain_id(),
    )?;
    assert!(txn_outputs.len() > 0);

    // Write data

    Ok((chain_state, net))
}

//
// pub fn prepare_customized_genesis(net: &ChainNetwork) -> ChainStateDB {
//     let chain_state = ChainStateDB::mock();
//     let genesis_txn = Genesis::build_genesis_transaction(net).unwrap();
//     Genesis::execute_genesis_txn(&chain_state, genesis_txn).unwrap();
//     chain_state
// }
