use starcoin_test_helper::executor::{get_sequence_number, prepare_genesis};
use starcoin_transaction_builder::vm2::DEFAULT_MAX_GAS_AMOUNT;
use starcoin_vm2_types::{
    account::Account,
    account_config::association_address,
    transaction::{
        authenticator::AccountPublicKey, DryRunTransaction, TransactionStatus,
    },
};
use starcoin_vm_runtime::data_cache::{AsMoveResolver, StateViewCache};
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;

#[test]
fn dry_run_transfer_materializes_delayed_fields() {
    let (chain_state, net) = prepare_genesis().expect("genesis should succeed");

    let receiver = Account::new();
    let sender = association_address();
    let seq_number = get_sequence_number(sender, &chain_state);

    let raw_txn = starcoin_transaction_builder::vm2::build_transfer_txn(
        sender,
        *receiver.address(),
        seq_number,
        1_000,
        1,
        DEFAULT_MAX_GAS_AMOUNT,
        1_000,
        net.chain_id().id().into(),
    );
    let (_, key) = &net.genesis_config2().association_key_pair;
    let public_key = AccountPublicKey::Multi(key.clone());
    let txn = DryRunTransaction {
        raw_txn,
        public_key,
    };

    let mut vm = StarcoinVM::new(None, &chain_state);
    let cache = StateViewCache::new(&chain_state);
    let (_status, output) = vm
        .dry_run_transaction(&cache.as_move_resolver(), txn)
        .expect("dry run should succeed");

    match output.status() {
        TransactionStatus::Keep(_) => {}
        status => panic!("dry run output discarded: {:?}", status),
    }
}
