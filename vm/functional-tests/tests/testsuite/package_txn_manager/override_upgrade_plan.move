// Test override upgrade plan
//! account: alice
//! account: bob

//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::Config;
use 0x1::Version;
use 0x1::PackageTxnManager;
use 0x1::Option;
fun main(account: signer) {
    Config::publish_new_config<Version::Version>(&account, Version::new_version(1));
    PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_two_phase(), Option::some<u64>(3));
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::submit_upgrade_plan(&account, copy hash, 1);
}
}

// check: EXECUTED

//! block-prologue
//! author: bob
//! block-time: 1
//! block-number: 1

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
fun main(account: signer) {
    let hash = x"2222222222222222";
    PackageTxnManager::submit_upgrade_plan(&account, copy hash, 2);
}
}

// check: EXECUTED

//! block-prologue
//! author: bob
//! block-time: 2
//! block-number: 2

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: signer) {
    let hash = x"2222222222222222";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: ABORTED

//! block-prologue
//! author: bob
//! block-time: 4
//! block-number: 3

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: signer) {
    let hash = x"2222222222222222";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: EXECUTED