//# init -n dev

//# faucet --addr alice

//# faucet --addr bob


//# run --signers alice
// default upgrade strategy is arbitrary
script {
use starcoin_framework::PackageTxnManager;
use starcoin_framework::signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(signer::address_of(&account), hash);
}
}

// check: EXECUTED

//# run --signers alice
script {
use starcoin_framework::on_chain_config;
use starcoin_framework::Version;
use starcoin_framework::PackageTxnManager;
use starcoin_framework::Option;
fun main(account: signer) {
    Config::publish_new_config<Version::Version>(&account, Version::new_version(1));
    PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_two_phase(), Option::some<u64>(0));
}
}

// check: EXECUTED

// two phase upgrade need to submit upgrade plan first.
//# run --signers alice
script {
use starcoin_framework::PackageTxnManager;
use starcoin_framework::signer;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::check_package_txn(signer::address_of(&account), hash);
}
}

// check: ABORTED

//# run --signers alice
script {
use starcoin_framework::PackageTxnManager;
fun main(account: signer) {
    let hash = x"1111111111111111";
    PackageTxnManager::submit_upgrade_plan_v2(&account, copy hash, 1, false);
}
}

// check: EXECUTED

//# run --signers alice
script {
use starcoin_framework::PackageTxnManager;
fun main(account: signer) {
    PackageTxnManager::cancel_upgrade_plan(&account);
}
}

// check: EXECUTED