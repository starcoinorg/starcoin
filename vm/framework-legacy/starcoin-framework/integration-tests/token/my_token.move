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

// check: EXECUTED

//# publish
module bob::HideToken {
    use alice::MyToken::MyToken;
    use StarcoinFramework::Token::Token;

    struct Collection has key, store { t: Token<MyToken>,}

    public fun hide(account: &signer, token: Token<MyToken>) {
        let b = Collection { t: token };
        move_to<Collection>(account, b);
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

// check: EXECUTED

//# run --signers bob
script {
    use StarcoinFramework::Account;
    use alice::MyToken::MyToken;

    fun main(account: signer) {
        Account::accept_token<MyToken>(account);
    }
}


//# run --signers alice
script {
    use StarcoinFramework::Account;
    use alice::MyToken::MyToken;

    fun main(account: signer) {
        Account::pay_from<MyToken>(&account, @bob, 10);
    }
}

//# run --signers bob
script {
    use StarcoinFramework::Account;
    use alice::MyToken::MyToken;
    use bob::HideToken;

    fun main(account: signer) {
        let token = Account::withdraw<MyToken>(&account, 10);
        HideToken::hide(&account, token);
    }
}