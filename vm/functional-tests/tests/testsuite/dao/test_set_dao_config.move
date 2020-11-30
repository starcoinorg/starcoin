//! account: alice
//! account: bob

//! sender: alice
module MyToken {
    use 0x1::Token;
    use 0x1::Dao;

    struct MyToken { }

    public fun init(account: &signer) {
        Token::register_token<MyToken>(
            account,
            3,
        );
        Dao::plugin<MyToken>(account, 60 * 1000, 60 * 60 * 1000, 4, 60 * 60 * 1000);
    }
}

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

//! new-transaction
//! sender: alice
script {
    use 0x1::Dao;
    use {{alice}}::MyToken::MyToken;
    use 0x1::Config;


    fun propose(signer: &signer) {
        let cap = Config::extract_modify_config_capability<Dao::DaoConfig<MyToken>>(
            signer
        );

        Dao::set_voting_delay<MyToken>(&mut cap, 30 * 1000);
        Dao::set_voting_period<MyToken>(&mut cap, 30 * 30 * 1000);
        Dao::set_voting_quorum_rate<MyToken>(&mut cap, 50);
        Dao::set_min_action_delay<MyToken>(&mut cap, 30 * 30 * 1000);

        Config::restore_modify_config_capability(cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use {{alice}}::MyToken::MyToken;
    use 0x1::ModifyDaoConfigProposal;

    fun test_plugin(signer: &signer) {
        ModifyDaoConfigProposal::plugin<MyToken>(signer); //ERR_NOT_AUTHORIZED
    }
}
// check: "Keep(ABORTED { code: 102658"