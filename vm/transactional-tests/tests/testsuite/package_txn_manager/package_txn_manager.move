//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# faucet --addr bob


//# run --signers alice
// default upgrade strategy is arbitrary
script {
use StarcoinFramework::PackageTxnManager;
use StarcoinFramework::Signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: EXECUTED

//# run --signers alice
script {
use StarcoinFramework::Config;
use StarcoinFramework::Version;
use StarcoinFramework::PackageTxnManager;
use StarcoinFramework::Option;
fun main(account: signer) {
    Config::publish_new_config<Version::Version>(&account, Version::new_version(1));
    PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_two_phase(), Option::some<u64>(2));
}
}

// check: EXECUTED

// two phase upgrade need to submit upgrade plan first.
//# run --signers alice
script {
use StarcoinFramework::PackageTxnManager;
use StarcoinFramework::Signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: ABORTED

//# run --signers alice
script {
use StarcoinFramework::PackageTxnManager;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::submit_upgrade_plan_v2(&account, copy hash, 1, false);
}
}

// check: EXECUTED

// package txn must wait after plan's active_after_number
//# run --signers alice
script {
use StarcoinFramework::PackageTxnManager;
use StarcoinFramework::Signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: ABORTED

//# block --author bob --timestamp 100000000000

//# run --signers alice
script {
use StarcoinFramework::PackageTxnManager;
use StarcoinFramework::Signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(Signer::address_of(&account), hash);
}
}

// check: EXECUTED

// cancel the upgrade plan
//# run --signers alice
script {
    use StarcoinFramework::PackageTxnManager;
    fun main(account: signer) {
        PackageTxnManager::cancel_upgrade_plan(&account);
    }
}

// check: EXECUTED

// cancel a none plan will report EUPGRADE_PLAN_IS_NONE
//# run --signers alice
script {
    use StarcoinFramework::PackageTxnManager;
    fun main(account: signer) {
        PackageTxnManager::cancel_upgrade_plan(&account);
    }
}

// check: "Keep(ABORTED { code: 26113"

//# run --signers alice
script {
    use StarcoinFramework::PackageTxnManager;
    use StarcoinFramework::Option;
    fun main(account: signer) {
        PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_arbitrary(), Option::some<u64>(0));
    }
}

// check: "Keep(ABORTED { code: 27143"

//# run --signers alice
script {
    use StarcoinFramework::PackageTxnManager;
    use StarcoinFramework::Option;
    fun main(account: signer) {
        PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_new_module(), Option::some<u64>(0));
    }
}

// check: EXECUTED



//# run --signers alice
script {
    use StarcoinFramework::PackageTxnManager;
    use StarcoinFramework::Option;
    fun main(account: signer) {
        PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_freeze(), Option::some<u64>(0));
    }
}
// check: EXECUTED

