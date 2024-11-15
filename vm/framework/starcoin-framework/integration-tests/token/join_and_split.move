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

// split and join
//# run --signers alice
script {
    use starcoin_framework::account;
    use starcoin_framework::Token;
    use alice::MyToken::{MyToken};
    fun main(account: signer) {
        let coin = Token::mint<MyToken>(&account, 10000);
        assert!(Token::value<MyToken>(&coin) == 10000, 8002);
        let (coin1, coin2) = Token::split<MyToken>(coin, 5000);
        assert!(Token::value<MyToken>(&coin1) == 5000, 8003);
        assert!(Token::value<MyToken>(&coin2) == 5000, 8004);
        let new_coin = Token::join<MyToken>(coin1, coin2);
        coin::deposit<MyToken>(&account, new_coin);
    }
}

// check: EXECUTED