use starcoin_crypto2::{ed25519::genesis_key_pair, HashValue};
use starcoin_executor::block_executor2::execute_genesis_transaction;
use starcoin_storage2::{storage::StorageInstance, Storage};
use starcoin_transaction2_builder::build_stdlib_package;
use starcoin_types2::account_config::CORE_CODE_ADDRESS;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::genesis_config::GenesisConfig;
use starcoin_vm2_types::transaction::{
    Package, RawUserTransaction, SignedUserTransaction, Transaction, TransactionPayload,
};
use std::sync::Arc;

fn build_genesis_transaction(
    chain_id: u8,
    genesis_config: &GenesisConfig,
) -> anyhow::Result<SignedUserTransaction> {
    let package = build_stdlib_package(chain_id.into(), genesis_config, None)?;
    build_genesis_transaction_with_package(chain_id, package)
}

fn build_genesis_transaction_with_package(
    chain_id: u8,
    package: Package,
) -> anyhow::Result<SignedUserTransaction> {
    let txn = RawUserTransaction::new_with_default_gas_token(
        CORE_CODE_ADDRESS,
        0,
        TransactionPayload::Package(package),
        0,
        0,
        1, // init to 1 to pass time check
        chain_id.into(),
    );
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();
    let sign_txn = txn.sign(&genesis_private_key, genesis_public_key)?;
    Ok(sign_txn.into_inner())
}

pub(crate) fn build_and_execute_genesis_transaction(
    chain_id: u8,
    genesis_config: &GenesisConfig,
) -> (SignedUserTransaction, HashValue) {
    let user_txn = build_genesis_transaction(chain_id, genesis_config).unwrap();

    let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance()).unwrap());
    let chain_state = ChainStateDB::new(storage.clone(), None);

    let txn = Transaction::UserTransaction(user_txn.clone());

    (
        user_txn,
        execute_genesis_transaction(&chain_state, txn)
            .unwrap()
            .txn_infos
            .pop()
            .expect("Execute output must exist.")
            .id(),
    )
}
