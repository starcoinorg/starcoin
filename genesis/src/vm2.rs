use starcoin_crypto2::{ed25519::genesis_key_pair, HashValue};
use starcoin_executor::block_executor2::execute_genesis_transaction;
use starcoin_storage2::{storage::StorageInstance, Storage};
use starcoin_transaction2_builder::build_stdlib_package;
use starcoin_types2::account_config::CORE_CODE_ADDRESS;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::{
    genesis_config::{BuiltinNetworkID, ChainId, ChainNetwork},
    transaction::{
        Package, RawUserTransaction, SignedUserTransaction, Transaction, TransactionPayload,
    },
};
use std::sync::Arc;

fn build_genesis_transaction(chain_id: u8) -> anyhow::Result<SignedUserTransaction> {
    let net = ChainNetwork::new_builtin(BuiltinNetworkID::try_from(ChainId::new(chain_id))?);
    let package = build_stdlib_package(&net, 0)?;
    build_genesis_transaction_with_package(net, package)
}

fn build_genesis_transaction_with_package(
    net: ChainNetwork,
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

pub(crate) fn build_and_execute_genesis_transaction(
    chain_id: u8,
) -> (SignedUserTransaction, HashValue) {
    let user_txn = build_genesis_transaction(chain_id).unwrap();

    let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance()).unwrap());
    let chain_state = ChainStateDB::new(storage.clone(), None);

    let txn = Transaction::UserTransaction(user_txn.clone());

    (
        user_txn,
        execute_genesis_transaction(&chain_state, txn, chain_id)
            .unwrap()
            .txn_infos
            .pop()
            .expect("Execute output must exist.")
            .id(),
    )
}
