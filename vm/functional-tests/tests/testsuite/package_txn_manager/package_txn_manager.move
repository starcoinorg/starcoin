// Test upgrade manager
//! account: alice
//! account: bob

// default upgrade strategy is arbitrary
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(account), {{alice}}, hash);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
fun main(account: &signer) {
    PackageTxnManager::update_module_upgrade_strategy(account, PackageTxnManager::STRATEGY_TWO_PHASE());
}
}

// check: EXECUTED

// two phase upgrade need to submit upgrade plan first.
//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(account), {{alice}}, hash);
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::submit_upgrade_plan(account, copy hash, 1);
}
}

// check: EXECUTED


// package txn must wait after plan's active_after_height
//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(account), {{alice}}, hash);
}
}

// check: ABORTED

//! block-prologue
//! proposer: bob
//! block-time: 100000000

//! new-transaction
//! sender: alice
//! expiration-time: 100000010
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(account), {{alice}}, hash);
}
}

// check: EXECUTED