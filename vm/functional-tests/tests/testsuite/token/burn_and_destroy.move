// Test user-defined token
//! account: alice
//! account: bob

//! sender: alice
module MyToken {
    use 0x1::Token;
    use 0x1::Signer;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer) {
        assert(Signer::address_of(account) == {{alice}}, 8000);

        Token::register_token<MyToken>(
            account,
            3,
        );
    }
}

// check: EXECUTED

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
    Account::deposit_to_self<MyToken>(account, coin);
}
}

// check: EXECUTED

// user burn
//! new-transaction
//! sender: alice
script {
use 0x1::Token;
use {{alice}}::MyToken::{MyToken};
fun test_withdraw_and_burn(account: &signer) {
    let cap = Token::remove_burn_capability<MyToken>(account);
    Token::add_burn_capability<MyToken>(account, cap);
}
}
// check: EXECUTED

// user burn
//! new-transaction
//! sender: alice
script {
use 0x1::Token;
use {{alice}}::MyToken::{MyToken};
use 0x1::Account;
fun test_withdraw_and_burn(account: &signer) {
    let market_cap = Token::market_cap<MyToken>();
    assert(market_cap == 10000, 8004);
    let token = Account::withdraw<MyToken>(account, 10000);
    let t1 = Token::withdraw<MyToken>(&mut token, 100);
    let t2 = Token::withdraw<MyToken>(&mut token, 10000); // amount is not enough
    Token::burn<MyToken>(account, token);
    Token::burn<MyToken>(account, t1);
    Token::burn<MyToken>(account, t2);
}
}

// check: "Keep(ABORTED { code: 26120"

//! new-transaction
//! sender: alice
script {
use 0x1::Token;
use {{alice}}::MyToken::MyToken;
fun test_mint_and_burn(account: &signer) {
    let old_market_cap = Token::market_cap<MyToken>();
    let amount = 100;
    let coin = Token::mint<MyToken>(account, amount);
    assert(Token::value<MyToken>(&coin) == amount, 8008);
    assert(Token::market_cap<MyToken>() == old_market_cap + amount, 8009);
    Token::burn<MyToken>(account, coin);
}
}

// check: EXECUTED

// destroy zero
//! new-transaction
//! sender: alice
script {
use 0x1::Token;
use {{alice}}::MyToken::{MyToken};
use 0x1::Account;
fun test_withdraw_and_burn(account: &signer) {
    let zero = Account::withdraw<MyToken>(account, 0);
    Token::destroy_zero<MyToken>(zero);
    let token = Account::withdraw<MyToken>(account, 10); //EDESTROY_TOKEN_NON_ZERO
    Token::destroy_zero<MyToken>(token);
}
}
// check: "Keep(ABORTED { code: 4097"

// destroy capability
//! new-transaction
//! sender: alice
script {
use 0x1::Token;
use {{alice}}::MyToken::{MyToken};
fun test_withdraw_and_burn(account: &signer) {
    let burn_cap = Token::remove_burn_capability<MyToken>(account);
    Token::destroy_burn_capability<MyToken>(burn_cap);
    let mint_cap = Token::remove_mint_capability<MyToken>(account);
    Token::destroy_mint_capability<MyToken>(mint_cap);
}
}
// check: EXECUTED