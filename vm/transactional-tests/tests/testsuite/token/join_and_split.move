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

// split and join
//# run --signers alice
script {
    use StarcoinFramework::Account;
    use StarcoinFramework::Token;
    use alice::MyToken::{MyToken};
    fun main(account: signer) {
        let coin = Token::mint<MyToken>(&account, 10000);
        assert!(Token::value<MyToken>(&coin) == 10000, 8002);
        let (coin1, coin2) = Token::split<MyToken>(coin, 5000);
        assert!(Token::value<MyToken>(&coin1) == 5000, 8003);
        assert!(Token::value<MyToken>(&coin2) == 5000, 8004);
        let new_coin = Token::join<MyToken>(coin1, coin2);
        Account::deposit_to_self<MyToken>(&account, new_coin);
    }
}

// check: EXECUTED