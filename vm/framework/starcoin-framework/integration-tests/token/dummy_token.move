//# init -n dev

//# faucet --addr alice


//# faucet --addr alice --amount 0

//# run --signers alice
script {
use starcoin_framework::DummyToken::{Self, DummyToken};
use starcoin_framework::Token;
use starcoin_framework::account;
use starcoin_framework::signer;
fun main(account: signer) {
    let account_address = signer::address_of(&account);
    let old_market_cap = Token::market_cap<DummyToken>();
    let amount = 100;
    let coin = DummyToken::mint(&account, amount);
    assert!(Token::value<DummyToken>(&coin) == amount, 1);
    assert!(Token::market_cap<DummyToken>() == old_market_cap + amount, 2);
    coin::deposit(&account, coin);
    assert!(account::balance<DummyToken>(account_address) == amount, 3);
}
}

// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::DummyToken::{Self, DummyToken};
    use starcoin_framework::Token;
    use starcoin_framework::account;
    use starcoin_framework::signer;
    fun test_burn(account: signer) {
        let account_address = signer::address_of(&account);
        let old_market_cap = Token::market_cap<DummyToken>();
        let amount = 100;
        let coin = DummyToken::mint(&account, amount);
        assert!(Token::value<DummyToken>(&coin) == amount, 1);
        assert!(Token::market_cap<DummyToken>() == old_market_cap + amount, 2);
        DummyToken::burn(coin);
        assert!(account::balance<DummyToken>(account_address) == amount, 3);
    }
}

// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::DummyToken::{Self, DummyToken};
    use starcoin_framework::Token;
    use starcoin_framework::account;
    use starcoin_framework::signer;
    fun amount_exceed_limit(account: signer) {
        let account_address = signer::address_of(&account);
        let old_market_cap = Token::market_cap<DummyToken>();
        let amount = 10001; // amount should < 10000
        let coin = DummyToken::mint(&account, amount);
        assert!(Token::value<DummyToken>(&coin) == amount, 1);
        assert!(Token::market_cap<DummyToken>() == old_market_cap + amount, 2);
        DummyToken::burn(coin);
        assert!(account::balance<DummyToken>(account_address) == amount, 3);
    }
}

// check: "Keep(ABORTED { code: 25863"