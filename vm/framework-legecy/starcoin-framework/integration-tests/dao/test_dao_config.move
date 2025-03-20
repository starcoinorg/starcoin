//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# publish

module alice::MyToken {
    use StarcoinFramework::Token;
    use StarcoinFramework::Dao;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer) {
        Token::register_token<MyToken>(
            account,
            3,
        );
        Dao::plugin<MyToken>(account, 60 * 1000, 60 * 60 * 1000, 4, 60 * 60 * 1000);
    }
}


//# run --signers alice

script {
    use alice::MyToken::{MyToken, Self};
    use StarcoinFramework::Account;
    use StarcoinFramework::Token;

    fun main(account: signer) {
        MyToken::init(&account);

        let market_cap = Token::market_cap<MyToken>();
        assert!(market_cap == 0, 8001);
        assert!(Token::is_registered_in<MyToken>(@alice), 8002);
        // Create 'Balance<TokenType>' resource under sender account, and init with zero
        Account::do_accept_token<MyToken>(&account);
    }
}

// check: EXECUTED


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Config;


    fun set_dao_config(signer: signer) {
        let cap = Config::extract_modify_config_capability<Dao::DaoConfig<MyToken>>(
            &signer
        );

        Dao::set_voting_delay<MyToken>(&mut cap, 30 * 1000);
        Dao::set_voting_period<MyToken>(&mut cap, 30 * 30 * 1000);
        Dao::set_voting_quorum_rate<MyToken>(&mut cap, 50);
        Dao::set_min_action_delay<MyToken>(&mut cap, 30 * 30 * 1000);

        Config::restore_modify_config_capability(cap);
    }
}
// check: EXECUTED


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Config;

    fun set_dao_config(signer: signer) {
        let cap = Config::extract_modify_config_capability<Dao::DaoConfig<MyToken>>(
            &signer
        );
        Dao::set_voting_delay<MyToken>(&mut cap, 0);
        Config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Config;

    fun set_dao_config(signer: signer) {
        let cap = Config::extract_modify_config_capability<Dao::DaoConfig<MyToken>>(
            &signer
        );
        Dao::set_voting_period<MyToken>(&mut cap, 0);
        Config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Config;

    fun set_dao_config(signer: signer) {
        let cap = Config::extract_modify_config_capability<Dao::DaoConfig<MyToken>>(
            &signer
        );
        Dao::set_voting_quorum_rate<MyToken>(&mut cap, 0);
        Config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 359943"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Config;

    fun set_dao_config(signer: signer) {
        let cap = Config::extract_modify_config_capability<Dao::DaoConfig<MyToken>>(
            &signer
        );
        Dao::set_min_action_delay<MyToken>(&mut cap, 0);
        Config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers bob

script {
    use alice::MyToken::MyToken;
    use StarcoinFramework::ModifyDaoConfigProposal;

    fun test_plugin(signer: signer) {
        ModifyDaoConfigProposal::plugin<MyToken>(&signer); //ERR_NOT_AUTHORIZED
    }
}
// check: "Keep(ABORTED { code: 102658"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Config;

    fun modify_dao_config(signer: signer) {
        let cap = Config::extract_modify_config_capability<Dao::DaoConfig<MyToken>>(
            &signer
        );
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 50;
        let min_action_delay = 30 * 30 * 1000;

        Dao::modify_dao_config<MyToken>(
            &mut cap,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );

        let voting_delay = 0;
        let voting_period = 0;
        let voting_quorum_rate = 0;
        let min_action_delay = 0;

        Dao::modify_dao_config<MyToken>(
            &mut cap,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );

        Config::restore_modify_config_capability(cap);
    }
}
// check: EXECUTED



//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Config;

    fun modify_dao_config(signer: signer) {
        let cap = Config::extract_modify_config_capability<Dao::DaoConfig<MyToken>>(
            &signer
        );
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 101; //ERR_QUORUM_RATE_INVALID
        let min_action_delay = 30 * 30 * 1000;

        Dao::modify_dao_config<MyToken>(
            &mut cap,
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );

        Config::restore_modify_config_capability(cap);
    }
}
// check: "Keep(ABORTED { code: 359943"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 0; //should > 0
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 50;
        let min_action_delay = 30 * 30 * 1000;

        Dao::new_dao_config<MyToken>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 30 * 1000;
        let voting_period = 0; //should > 0
        let voting_quorum_rate = 50;
        let min_action_delay = 30 * 30 * 1000;

        Dao::new_dao_config<MyToken>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 0; //should > 0
        let min_action_delay = 30 * 30 * 1000;

        Dao::new_dao_config<MyToken>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 101; //should <= 100
        let min_action_delay = 30 * 30 * 1000;

        Dao::new_dao_config<MyToken>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"


//# run --signers alice

script {
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;

    fun new_dao_config_failed(_signer: signer) {
        let voting_delay = 30 * 1000;
        let voting_period = 30 * 30 * 1000;
        let voting_quorum_rate = 50;
        let min_action_delay = 0; //should > 0

        Dao::new_dao_config<MyToken>(
            voting_delay,
            voting_period,
            voting_quorum_rate,
            min_action_delay
        );
    }
}
// check: "Keep(ABORTED { code: 360199"