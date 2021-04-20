// Test upgrade manager
//! account: alice
//! account: bob

// default upgrade strategy is arbitrary
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
    PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_two_phase(), Option::some<u64>(2));
}
}

// check: EXECUTED

// two phase upgrade need to submit upgrade plan first.
//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::submit_upgrade_plan_v2(&account, copy hash, 1, false);
}
}

// check: EXECUTED

// package txn must wait after plan's active_after_number
//! new-transaction
//! sender: alice
script {
use 0x1::PackageTxnManager;
use 0x1::Signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
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
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: EXECUTED

// cancel the upgrade plan
//! new-transaction
//! sender: alice
script {
    use 0x1::PackageTxnManager;
    fun main(account: signer) {
        PackageTxnManager::cancel_upgrade_plan(&account);
    }
}

// check: EXECUTED

// cancel a none plan will report EUPGRADE_PLAN_IS_NONE
//! new-transaction
//! sender: alice
script {
    use 0x1::PackageTxnManager;
    fun main(account: signer) {
        PackageTxnManager::cancel_upgrade_plan(&account);
    }
}

// check: "Keep(ABORTED { code: 26113"

//! new-transaction
//! sender: alice
script {
    use 0x1::PackageTxnManager;
    use 0x1::Option;
    fun main(account: signer) {
        PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_arbitrary(), Option::some<u64>(0));
    }
}

// check: "Keep(ABORTED { code: 27143"

//! new-transaction
//! sender: alice
script {
    use 0x1::PackageTxnManager;
    use 0x1::Option;
    fun main(account: signer) {
        PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_new_module(), Option::some<u64>(0));
    }
}

// check: EXECUTED


//! new-transaction
//! sender: alice
script {
    use 0x1::PackageTxnManager;
    use 0x1::Option;
    fun main(account: signer) {
        PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_freeze(), Option::some<u64>(0));
    }
}
// check: EXECUTED

