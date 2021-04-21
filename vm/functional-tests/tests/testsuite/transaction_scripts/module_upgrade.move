//! account: alice
//! account: bob

//! sender: alice
module MyToken {
    use 0x1::Token;
    use 0x1::Dao;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer) {
        Token::register_token<MyToken>(
            account,
            3,
        );
        Dao::plugin<MyToken>(account, 60 * 1000, 60 * 60 * 1000, 4, 60 * 60 * 1000);
    }
}
// check: gas_used
// check: 7800

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 1000

//! new-transaction
//! sender: alice
script {
    use {{alice}}::MyToken::{MyToken, Self};
    use 0x1::Account;
    use 0x1::Token;

    fun main(account: signer) {
        MyToken::init(&account);
        Account::do_accept_token<MyToken>(&account);
        let coin = Token::mint<MyToken>(&account, 10000);
        Account::deposit_to_self<MyToken>(&account, coin)
    }
}

// check: EXECUTED
// check: gas_used
// check: 1400132

//! new-transaction
//! sender: alice
script {
    use 0x1::Config;
    use 0x1::Version;
    use 0x1::PackageTxnManager;
    use 0x1::Option;

    fun update_module_upgrade_strategy(account: signer) {
        Config::publish_new_config<Version::Version>(&account, Version::new_version(1));
        PackageTxnManager::update_module_upgrade_strategy(&account, PackageTxnManager::get_strategy_two_phase(), Option::some<u64>(1));
    }
}
// check: EXECUTED
// check: gas_used
// check: 599685
//
//! new-transaction
//! sender: alice
script {
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::PackageTxnManager;
    use {{alice}}::MyToken::MyToken;

    fun plugin(account: signer) {
        let upgrade_plan_cap = PackageTxnManager::extract_submit_upgrade_plan_cap(&account);
        UpgradeModuleDaoProposal::plugin<MyToken>(&account, upgrade_plan_cap);
    }
}

// check: EXECUTED
// check: gas_used
// check: 58927

//! new-transaction
//! sender: alice
script {
    use 0x1::UpgradeModuleDaoProposal;
    use {{alice}}::MyToken::MyToken;

    fun propose_module_upgrade(account: signer) {
        let module_address = {{alice}};
        let package_hash = x"1111111111111111";
        let version = 1;
        let exec_delay = 60 * 60 * 1000;
        UpgradeModuleDaoProposal::propose_module_upgrade_v2<MyToken>(
            &account,
            module_address,
            copy package_hash,
            version,
            exec_delay,
            false,
        );
    }
}
// check: EXECUTED
// check: gas_used
// check: 207001

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 3601000

//! new-transaction
//! sender: alice
//! args: {{alice}}, 0, true, 500u128
script {
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::Dao;
    use 0x1::Account;
    use {{alice}}::MyToken::MyToken;

    fun cast_vote(
        signer: signer,
        proposer_address: address,
        proposal_id: u64,
        agree: bool,
        votes: u128,
    ) {
        let stake = Account::withdraw<MyToken>(&signer, votes);
        Dao::cast_vote<MyToken, UpgradeModuleDaoProposal::UpgradeModuleV2>(&signer, proposer_address, proposal_id, stake, agree);
    }
}
// check: EXECUTED
// check: gas_used
// check: 156792

//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 3662000

//! new-transaction
//! sender: alice
//! args: {{alice}}, 0
script {
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::Dao;
    use {{alice}}::MyToken::MyToken;

    fun queue_proposal_action(_signer: signer,
        proposer_address: address,
        proposal_id: u64
    ) {
        Dao::queue_proposal_action<MyToken, UpgradeModuleDaoProposal::UpgradeModuleV2>(proposer_address, proposal_id);
    }
}
// check: EXECUTED
// check: gas_used
// check: 47257

//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 7262000


//! new-transaction
//! sender: alice
//! args: {{alice}}, 0
script {
    use 0x1::UpgradeModuleDaoProposal;
    use {{alice}}::MyToken::MyToken;

    fun submit_module_upgrade_plan(_account: signer, proposer_address: address, proposal_id: u64) {
        UpgradeModuleDaoProposal::submit_module_upgrade_plan<MyToken>(proposer_address, proposal_id);
    }
}
// check: EXECUTED
// check: gas_used
// check: 123307
