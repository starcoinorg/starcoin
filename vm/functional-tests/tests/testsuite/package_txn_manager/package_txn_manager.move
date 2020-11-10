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
    PackageTxnManager::check_package_txn(Signer::address_of(account), hash);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::Config;
use 0x1::Version;
use 0x1::PackageTxnManager;
fun main(account: &signer) {
    Config::publish_new_config<Version::Version>(account, Version::new_version(1));
    PackageTxnManager::update_module_upgrade_strategy(account, PackageTxnManager::get_strategy_two_phase());
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
    PackageTxnManager::check_package_txn(Signer::address_of(account), hash);
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::submit_upgrade_plan(account, copy hash, 1, 1);
}
}

// check: EXECUTED


// package txn must wait after plan's active_after_number
//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(account), hash);
}
}

// check: ABORTED

//! block-prologue
//! author: bob
//! block-time: 100000000000
//! block-number: 1

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(account), hash);
}
}

// check: EXECUTED