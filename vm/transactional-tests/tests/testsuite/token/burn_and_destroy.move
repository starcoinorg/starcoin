//# init -n dev

//# faucet --addr alice

//# faucet --addr bob


//# publish
module alice::MyToken {
    use StarcoinFramework::Token;
    use StarcoinFramework::Signer;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer) {
        assert!(Signer::address_of(account) == @alice, 8000);

        Token::register_token<MyToken>(
            account,
            3,
        );
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
    Account::deposit_to_self<MyToken>(&account, coin);
}
}

//# run --signers alice
script {
use StarcoinFramework::Token;
use alice::MyToken::{MyToken};
fun test_withdraw_and_burn(account: signer) {
    let cap = Token::remove_burn_capability<MyToken>(&account);
    Token::add_burn_capability<MyToken>(&account, cap);
}
}

//# run --signers alice
script {
use StarcoinFramework::Token;
use alice::MyToken::{MyToken};
use StarcoinFramework::Account;
fun test_withdraw_and_burn(account: signer) {
    let market_cap = Token::market_cap<MyToken>();
    assert!(market_cap == 10000, 8004);
    let token = Account::withdraw<MyToken>(&account, 10000);
    let t1 = Token::withdraw<MyToken>(&mut token, 100);
    let t2 = Token::withdraw<MyToken>(&mut token, 10000); // amount is not enough
    Token::burn<MyToken>(&account, token);
    Token::burn<MyToken>(&account, t1);
    Token::burn<MyToken>(&account, t2);
}
}

//# run --signers alice
script {
use StarcoinFramework::Token;
use alice::MyToken::MyToken;
fun test_mint_and_burn(account: signer) {
    let old_market_cap = Token::market_cap<MyToken>();
    let amount = 100;
    let coin = Token::mint<MyToken>(&account, amount);
    assert!(Token::value<MyToken>(&coin) == amount, 8008);
    assert!(Token::market_cap<MyToken>() == old_market_cap + amount, 8009);
    Token::burn<MyToken>(&account, coin);
}
}

//# run --signers alice
script {
use StarcoinFramework::Token;
use alice::MyToken::{MyToken};
use StarcoinFramework::Account;
fun test_withdraw_and_burn(account: signer) {
    let zero = Account::withdraw<MyToken>(&account, 0);
    Token::destroy_zero<MyToken>(zero);
    let token = Account::withdraw<MyToken>(&account, 10); //EDESTROY_TOKEN_NON_ZERO
    Token::destroy_zero<MyToken>(token);
}
}

//# run --signers alice
script {
use StarcoinFramework::Token;
use alice::MyToken::{MyToken};
fun test_withdraw_and_burn(account: signer) {
    let burn_cap = Token::remove_burn_capability<MyToken>(&account);
    Token::destroy_burn_capability<MyToken>(burn_cap);
    let mint_cap = Token::remove_mint_capability<MyToken>(&account);
    Token::destroy_mint_capability<MyToken>(mint_cap);
}
}
