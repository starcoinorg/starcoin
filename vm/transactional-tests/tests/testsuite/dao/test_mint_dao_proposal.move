//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# publish

module alice::MyToken {
    use StarcoinFramework::Token;
    use StarcoinFramework::MintDaoProposal;
    use StarcoinFramework::Dao;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer) {
        Token::register_token<MyToken>(
            account,
            3,
        );
    }

    public fun delegate_to_dao(account: &signer) {
        Dao::plugin<MyToken>(account, 60 * 1000, 60 * 60 * 1000, 4, 60 * 60 * 1000);
        MintDaoProposal::plugin<MyToken>(account);
    }
}

//# block --author 0x1 --timestamp 86400000


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

// issuer mint

//# run --signers alice

script {
    use StarcoinFramework::Account;
    use StarcoinFramework::Token;
    use alice::MyToken::{MyToken};
    fun main(account: signer) {
    // mint 100 coins and check that the market cap increases appropriately
        let old_market_cap = Token::market_cap<MyToken>();
        let coin = Token::mint<MyToken>(&account, 10000);
        assert!(Token::value<MyToken>(&coin) == 10000, 8002);
        assert!(Token::market_cap<MyToken>() == old_market_cap + 10000, 8003);
        Account::deposit_to_self<MyToken>(&account, coin)
    }
}


//# run --signers bob

script {
    use alice::MyToken::MyToken;
    use StarcoinFramework::Account;

    fun accept_token(account: signer) {
        Account::do_accept_token<MyToken>(&account);
    }
}

// check: EXECUTED


//# run --signers alice


script {
    use StarcoinFramework::Account;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Signer;

    fun transfer_some_token_to_bob(signer: signer) {
        let balance = Account::balance<MyToken>(Signer::address_of(&signer));
        Account::pay_from<MyToken>(&signer, @bob, balance / 2);
    }
}
// check: EXECUTED


//# run --signers bob

script {
    use alice::MyToken;

    fun delegate(account: signer) {
        MyToken::delegate_to_dao(&account);
    }
}
// check: "Keep(ABORTED { code: 358658"


//# run --signers bob

script {
    use alice::MyToken::MyToken;
    use StarcoinFramework::MintDaoProposal;

    fun test_plugin_fail(account: signer) {
        MintDaoProposal::plugin<MyToken>(&account); //ERR_NOT_AUTHORIZED
    }
}

// check: "Keep(ABORTED { code: 102658"


//# run --signers alice

script {
    use alice::MyToken;

    fun delegate(account: signer) {
        MyToken::delegate_to_dao(&account);
    }
}

// check: EXECUTED


//# run --signers alice

script {
    use StarcoinFramework::MintDaoProposal;
    use alice::MyToken::MyToken;

    fun propose(signer: signer) {
        MintDaoProposal::propose_mint_to<MyToken>(&signer, @alice, 1000000, 0);
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 87000000



//# run --signers bob


script {
    use StarcoinFramework::MintDaoProposal;
    use alice::MyToken::MyToken;
    use StarcoinFramework::Account;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Dao;

    fun vote(signer: signer) {
        let balance = Account::balance<MyToken>(Signer::address_of(&signer));
        let balance = Account::withdraw<MyToken>(&signer, balance);
        Dao::cast_vote<MyToken, MintDaoProposal::MintToken>(&signer, @alice, 0, balance, true);
    }
}
// check: EXECUTED


//# block --author 0x1 --timestamp 750000000

//# run --signers bob


script {
    use StarcoinFramework::MintDaoProposal;
    use StarcoinFramework::Account;
    use StarcoinFramework::Dao;
    use alice::MyToken::MyToken;

    fun queue_proposal(signer: signer) {
        let state = Dao::proposal_state<MyToken, MintDaoProposal::MintToken>(@alice, 0);
        assert!(state == 4, (state as u64));
        {
            let token = Dao::unstake_votes<MyToken, MintDaoProposal::MintToken>(&signer, @alice, 0);
            Account::deposit_to_self(&signer, token);
        };
        Dao::queue_proposal_action<MyToken, MintDaoProposal::MintToken>(@alice, 0);
        let state = Dao::proposal_state<MyToken, MintDaoProposal::MintToken>(@alice, 0);
        assert!(state == 5, (state as u64));
    }
}
// check: EXECUTED

//# block --author 0x1 --timestamp 850000000



//# run --signers bob


script {
    use StarcoinFramework::MintDaoProposal;
    use StarcoinFramework::Dao;
    use StarcoinFramework::Account;
    use alice::MyToken::MyToken;

    fun execute_proposal_action(_signer: signer) {
        let old_balance = Account::balance<MyToken>(@alice);
        let state = Dao::proposal_state<MyToken, MintDaoProposal::MintToken>(@alice, 0);
        assert!(state == 6, (state as u64));
        MintDaoProposal::execute_mint_proposal<MyToken>(@alice, 0);
        let balance = Account::balance<MyToken>(@alice);
        assert!(balance ==  old_balance + 1000000, 1001);
    }
}
// check: EXECUTED


