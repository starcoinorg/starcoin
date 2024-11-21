//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# run --signers alice
script {
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::signer;

    fun main(account: signer) {
        let hash = x"1111111111111111";
        stc_transaction_package_validation::check_package_txn(signer::address_of(&account), hash);
    }
}

// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::on_chain_config;
    use starcoin_framework::stc_version;
    use starcoin_framework::stc_transaction_package_validation;
    use std::option;

    fun main(account: signer) {
        on_chain_config::publish_new_config<stc_version::Version>(&account, stc_version::new_version(1));
        stc_transaction_package_validation::update_module_upgrade_strategy(
            &account,
            stc_transaction_package_validation::get_strategy_two_phase(),
            option::some<u64>(3)
        );
    }
}

// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::stc_transaction_package_validation;

    fun main(account: signer) {
        let hash = x"1111111111111111";
        stc_transaction_package_validation::submit_upgrade_plan_v2(&account, copy hash, 1, false);
    }
}

// check: EXECUTED

//# block --author bob

//# run --signers alice
script {
    use starcoin_framework::stc_transaction_package_validation;

    fun main(account: signer) {
        let hash = x"2222222222222222";
        stc_transaction_package_validation::submit_upgrade_plan_v2(&account, copy hash, 2, false);
    }
}

// check: EXECUTED

//# block --author bob

//# run --signers alice
script {
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::signer;

    fun main(account: signer) {
        let hash = x"2222222222222222";
        stc_transaction_package_validation::check_package_txn(signer::address_of(&account), hash);
    }
}

// check: ABORTED

//! block-prologue
//! author: bob
//! block-time: 4
//! block-number: 3

//# block --author bob


//# run --signers alice
script {
    use starcoin_framework::stc_transaction_package_validation;
    use starcoin_framework::signer;

    fun main(account: signer) {
        let hash = x"2222222222222222";
        stc_transaction_package_validation::check_package_txn(signer::address_of(&account), hash);
    }
}

// check: EXECUTED