// Test upgrade manager
//! account: alice
//! account: bob

// default upgrade strategy is arbitrary
//! sender: alice
script {
use 0x1::UpgradeManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    UpgradeManager::check_module_upgrade(Signer::address_of(account), {{alice}}, hash);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::UpgradeManager;
fun main(account: &signer) {
    UpgradeManager::update_module_upgrade_strategy(account, UpgradeManager::STRATEGY_TWO_PHASE());
}
}

// check: EXECUTED

// two phase upgrade need to submit upgrade plan first.
//! new-transaction
//! sender: alice
script {
use 0x1::UpgradeManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    UpgradeManager::check_module_upgrade(Signer::address_of(account), {{alice}}, hash);
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
script {
use 0x1::UpgradeManager;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    UpgradeManager::submit_upgrade_plan(account, copy hash, 1);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::UpgradeManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    UpgradeManager::check_module_upgrade(Signer::address_of(account), {{alice}}, hash);
}
}

// check: ABORT

//! block-prologue
//! proposer: bob
//! block-time: 100000000

//! new-transaction
//! sender: alice
script {
use 0x1::UpgradeManager;
use 0x1::Signer;
fun main(account: &signer) {
    let hash = x"1111111111111111";
    UpgradeManager::check_module_upgrade(Signer::address_of(account), {{alice}}, hash);
}
}

// check: EXECUTED