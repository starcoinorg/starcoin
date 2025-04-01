use starcoin_vm2_genesis::{build_genesis_transaction, execute_genesis_transaction};
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::genesis_config::ChainNetwork;
use starcoin_vm2_types::transaction::Transaction;

pub fn prepare_genesis() -> anyhow::Result<(ChainStateDB, ChainNetwork)> {
    let net = ChainNetwork::new_test();
    let chain_state = ChainStateDB::mock();
    let genesis_txn = build_genesis_transaction(&net).unwrap();
    // execute_genesis_txn(&chain_state, genesis_txn).unwrap();
    execute_genesis_transaction(&chain_state, Transaction::UserTransaction(genesis_txn))?;
    Ok((chain_state, net))
}

pub fn prepare_customized_genesis(net: &ChainNetwork) -> anyhow::Result<ChainStateDB> {
    let chain_state = ChainStateDB::mock();
    let genesis_txn = build_genesis_transaction(net).unwrap();
    // execute_genesis_txn(&chain_state, genesis_txn).unwrap();
    execute_genesis_transaction(&chain_state, Transaction::UserTransaction(genesis_txn))?;
    Ok(chain_state)
}

// pub fn prepare_genesis() -> anyhow::Result<(ChainStateDB, ChainNetwork)> {
//     let net = ChainNetwork::new_test();
//     let chain_state = ChainStateDB::mock();
//     let genesis_txn = build_genesis_transaction(&net).unwrap();
//
//     // Execute with vm 2
//     // let storage = Arc::new(Storage2::new(StorageInstance2::new_cache_instance())?);
//     let statedb = ChainStateDB::mock();
//     let txn_outputs = execute_block_transactions_with_chain_id(
//         &statedb,
//         vec![Transaction::UserTransaction(genesis_txn)],
//         0,
//         None,
//         &net.chain_id(),
//     )?;
//     assert!(txn_outputs.len() > 0);
//
//     // Write data
//
//     Ok((chain_state, net))
// }
