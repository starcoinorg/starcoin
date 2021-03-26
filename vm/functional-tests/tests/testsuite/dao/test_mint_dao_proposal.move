//! account: alice
//! account: bob

//! sender: alice
module MyToken {
    use 0x1::Token;
    use 0x1::MintDaoProposal;
    use 0x1::Dao;

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

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 86400000

//! new-transaction
//! sender: alice
script {
    use {{alice}}::MyToken::{MyToken, Self};
    use 0x1::Account;
    use 0x1::Token;

    fun main(account: signer) {
        MyToken::init(&account);

        let market_cap = Token::market_cap<MyToken>();
        assert(market_cap == 0, 8001);
        assert(Token::is_registered_in<MyToken>({{alice}}), 8002);
        // Create 'Balance<TokenType>' resource under sender account, and init with zero
        Account::do_accept_token<MyToken>(&account);
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
    fun main(account: signer) {
    // mint 100 coins and check that the market cap increases appropriately
        let old_market_cap = Token::market_cap<MyToken>();
        let coin = Token::mint<MyToken>(&account, 10000);
        assert(Token::value<MyToken>(&coin) == 10000, 8002);
        assert(Token::market_cap<MyToken>() == old_market_cap + 10000, 8003);
        Account::deposit_to_self<MyToken>(&account, coin)
    }
}

//! new-transaction
//! sender: bob
script {
    use {{alice}}::MyToken::MyToken;
    use 0x1::Account;

    fun accept_token(account: signer) {
        Account::do_accept_token<MyToken>(&account);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::Account;
    use {{alice}}::MyToken::MyToken;
    use 0x1::Signer;

    fun transfer_some_token_to_bob(signer: signer) {
        let balance = Account::balance<MyToken>(Signer::address_of(&signer));
        Account::pay_from<MyToken>(&signer, {{bob}}, balance / 2);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use {{alice}}::MyToken;

    fun delegate(account: signer) {
        MyToken::delegate_to_dao(&account);
    }
}
// check: "Keep(ABORTED { code: 358658"

//! new-transaction
//! sender: bob
script {
    use {{alice}}::MyToken::MyToken;
    use 0x1::MintDaoProposal;

    fun test_plugin_fail(account: signer) {
        MintDaoProposal::plugin<MyToken>(&account); //ERR_NOT_AUTHORIZED
    }
}

// check: "Keep(ABORTED { code: 102658"

//! new-transaction
//! sender: alice
script {
    use {{alice}}::MyToken;

    fun delegate(account: signer) {
        MyToken::delegate_to_dao(&account);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::MintDaoProposal;
    use {{alice}}::MyToken::MyToken;

    fun propose(signer: signer) {
        MintDaoProposal::propose_mint_to<MyToken>(&signer, {{alice}}, 1000000, 0);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 87000000


//! new-transaction
//! sender: bob

script {
    use 0x1::MintDaoProposal;
    use {{alice}}::MyToken::MyToken;
    use 0x1::Account;
    use 0x1::Signer;
    use 0x1::Dao;

    fun vote(signer: signer) {
        let balance = Account::balance<MyToken>(Signer::address_of(&signer));
        let balance = Account::withdraw<MyToken>(&signer, balance);
        Dao::cast_vote<MyToken, MintDaoProposal::MintToken>(&signer, {{alice}}, 0, balance, true);
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 180000000

//! new-transaction
//! sender: bob

script {
    use 0x1::MintDaoProposal;
    use 0x1::Account;
    use 0x1::Dao;
    use {{alice}}::MyToken::MyToken;

    fun queue_proposal(signer: signer) {
        let state = Dao::proposal_state<MyToken, MintDaoProposal::MintToken>({{alice}}, 0);
        assert(state == 4, (state as u64));
        {
            let token = Dao::unstake_votes<MyToken, MintDaoProposal::MintToken>(&signer, {{alice}}, 0);
            Account::deposit_to_self(&signer, token);
        };
        Dao::queue_proposal_action<MyToken, MintDaoProposal::MintToken>({{alice}}, 0);
        let state = Dao::proposal_state<MyToken, MintDaoProposal::MintToken>({{alice}}, 0);
        assert(state == 5, (state as u64));
    }
}
// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 250000000


//! new-transaction
//! sender: bob

script {
    use 0x1::MintDaoProposal;
    use 0x1::Dao;
    use 0x1::Account;
    use {{alice}}::MyToken::MyToken;

    fun execute_proposal_action(_signer: signer) {
        let old_balance = Account::balance<MyToken>({{alice}});
        let state = Dao::proposal_state<MyToken, MintDaoProposal::MintToken>({{alice}}, 0);
        assert(state == 6, (state as u64));
        MintDaoProposal::execute_mint_proposal<MyToken>({{alice}}, 0);
        let balance = Account::balance<MyToken>({{alice}});
        assert(balance ==  old_balance + 1000000, 1001);
    }
}
// check: EXECUTED


