//# init -n dev

//# faucet --addr alice

//# faucet --addr bob


//# publish
module alice::MyToken {
    use starcoin_framework::Token;
    use starcoin_framework::signer;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer) {
        assert!(signer::address_of(account) == @alice, 8000);

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
    use starcoin_framework::Token::Token;

    struct Collection has key, store { t: Token<MyToken>,}

    public fun hide(account: &signer, token: Token<MyToken>) {
        let b = Collection { t: token };
        move_to<Collection>(account, b);
    }
}


//# run --signers alice
script {
use alice::MyToken::{MyToken, Self};
use starcoin_framework::account;
use starcoin_framework::Token;

fun main(account: signer) {
    MyToken::init(&account);

    let market_cap = Token::market_cap<MyToken>();
    assert!(market_cap == 0, 8001);
    assert!(Token::is_registered_in<MyToken>(@alice), 8002);
    // Create 'Balance<TokenType>' resource under sender account, and init with zero
    account::do_accept_token<MyToken>(&account);
}
}

// check: EXECUTED


//# run --signers alice
script {
use starcoin_framework::account;
use starcoin_framework::Token;
use alice::MyToken::{MyToken};
fun main(account: signer) {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Token::market_cap<MyToken>();
    let coin = Token::mint<MyToken>(&account, 10000);
    assert!(Token::value<MyToken>(&coin) == 10000, 8002);
    assert!(Token::market_cap<MyToken>() == old_market_cap + 10000, 8003);
    coin::deposit<MyToken>(&account, coin);
}
}

// check: EXECUTED

//# run --signers bob
script {
    use starcoin_framework::account;
    use alice::MyToken::MyToken;

    fun main(account: signer) {
        account::accept_token<MyToken>(account);
    }
}


//# run --signers alice
script {
    use starcoin_framework::account;
    use alice::MyToken::MyToken;

    fun main(account: signer) {
        coin::transfer<MyToken>(&account, @bob, 10);
    }
}

//# run --signers bob
script {
    use starcoin_framework::account;
    use alice::MyToken::MyToken;
    use bob::HideToken;

    fun main(account: signer) {
        let token = coin::withdraw<MyToken>(&account, 10);
        HideToken::hide(&account, token);
    }
}