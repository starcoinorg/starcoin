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

    fun main(account: &signer) {
        MyToken::init(account);

        let market_cap = Token::market_cap<MyToken>();
        assert(market_cap == 0, 8001);
        assert(Token::is_registered_in<MyToken>({{alice}}), 8002);
        // Create 'Balance<TokenType>' resource under sender account, and init with zero
        Account::accept_token<MyToken>(account);
    }
}

// check: EXECUTED


// issuer mint
//! new-transaction
//! sender: alice
script {
    use 0x1::Account;
    use 0x1::Token;
    use {{alice}}::MyToken::{MyToken};
    fun main(account: &signer) {
        // mint 100 coins and check that the market cap increases appropriately
        let old_market_cap = Token::market_cap<MyToken>();
        let coin = Token::mint<MyToken>(account, 10000);
        assert(Token::value<MyToken>(&coin) == 10000, 8002);
        assert(Token::market_cap<MyToken>() == old_market_cap + 10000, 8003);
        Account::deposit_to_self<MyToken>(account, coin)
    }
}

// default upgrade strategy is arbitrary
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

//! new-transaction
//! sender: alice
script {
    use 0x1::Config;
    use 0x1::Version;
    use 0x1::PackageTxnManager;
    use 0x1::Option;
    fun main(account: &signer) {
        Config::publish_new_config<Version::Version>(account, Version::new_version(1));
        PackageTxnManager::update_module_upgrade_strategy(account, PackageTxnManager::get_strategy_two_phase(), Option::some<u64>(0));
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::PackageTxnManager;
    use 0x1::STC::STC;

    fun test_plugin_fail(account: &signer) {
        let upgrade_plan_cap = PackageTxnManager::extract_submit_upgrade_plan_cap(account);
        UpgradeModuleDaoProposal::plugin<STC>(account, upgrade_plan_cap); //ERR_NOT_AUTHORIZED
    }
}

// check: "Keep(ABORTED { code: 102658"

//! new-transaction
//! sender: alice
script {
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::PackageTxnManager;
    use {{alice}}::MyToken::MyToken;


fun test_plugin(account: &signer) {
        let upgrade_plan_cap = PackageTxnManager::extract_submit_upgrade_plan_cap(account);
        UpgradeModuleDaoProposal::plugin<MyToken>(account, upgrade_plan_cap);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::STC::STC;

    fun test_propose_fail(account: &signer) {
        let module_address = {{alice}};
        let package_hash = x"1111111111111111";
        let version = 1;
        let exec_delay = 1;
        UpgradeModuleDaoProposal::propose_module_upgrade<STC>(
            account,
            module_address, //ERR_ADDRESS_MISSMATCH
            copy package_hash,
            version,
            exec_delay
        );
    }
}
// check: "Keep(ABORTED { code: 102916"

//! new-transaction
//! sender: alice
script {
    use 0x1::UpgradeModuleDaoProposal;
    use {{alice}}::MyToken::MyToken;

    fun test_propose(account: &signer) {
        let module_address = {{alice}};
        let package_hash = x"1111111111111111";
        let version = 1;
        let exec_delay = 60 * 60 * 1000;
        UpgradeModuleDaoProposal::propose_module_upgrade<MyToken>(
            account,
            module_address,
            copy package_hash,
            version,
            exec_delay
        );
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 3601000

//! new-transaction
//! sender: alice

script {
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::Dao;
    use {{alice}}::MyToken::MyToken;
    use 0x1::Account;
    use 0x1::Signer;

    fun vote_proposal(signer: &signer) {
        let proposal_id = 0;
        let state = Dao::proposal_state<MyToken, UpgradeModuleDaoProposal::UpgradeModule>({{alice}}, proposal_id);
        assert(state == 2, (state as u64));
        let balance = Account::balance<MyToken>(Signer::address_of(signer));
        let balance = Account::withdraw<MyToken>(signer, balance / 2);
        Dao::cast_vote<MyToken, UpgradeModuleDaoProposal::UpgradeModule>(signer, {{alice}}, proposal_id, balance, true);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 3662000

//! new-transaction
//! sender: alice

script {
    use 0x1::UpgradeModuleDaoProposal;
    use 0x1::Dao;
    use {{alice}}::MyToken::MyToken;

    fun queue_proposal(_signer: &signer) {
        let proposal_id = 0;
        let state = Dao::proposal_state<MyToken, UpgradeModuleDaoProposal::UpgradeModule>({{alice}}, proposal_id);
        assert(state == 4, (state as u64));
        Dao::queue_proposal_action<MyToken, UpgradeModuleDaoProposal::UpgradeModule>({{alice}}, proposal_id);
        let state = Dao::proposal_state<MyToken, UpgradeModuleDaoProposal::UpgradeModule>({{alice}}, proposal_id);
        assert(state == 5, (state as u64));
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 7262000


//! new-transaction
//! sender: alice
script {
    use 0x1::UpgradeModuleDaoProposal;
    use {{alice}}::MyToken::MyToken;
    use 0x1::Dao;

    fun test_submit_plan(_account: &signer) {
        let proposal_id = 0;
        let proposer_address = {{alice}};
        let state = Dao::proposal_state<MyToken, UpgradeModuleDaoProposal::UpgradeModule>(proposer_address, proposal_id);
        assert(state == 6, (state as u64));
        UpgradeModuleDaoProposal::submit_module_upgrade_plan<MyToken>(proposer_address, proposal_id);
    }
}
// check: EXECUTED

