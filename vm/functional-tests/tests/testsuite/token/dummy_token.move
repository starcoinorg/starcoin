// Test the mint flow
//! account: alice, 0 0x1::STC::STC

// Minting from a privileged account should work
//! sender: alice
script {
use 0x1::DummyToken::{Self, DummyToken};
use 0x1::Token;
use 0x1::Account;
use 0x1::Signer;
fun main(account: &signer) {
    let account_address = Signer::address_of(account);
    let old_market_cap = Token::market_cap<DummyToken>();
    let amount = 100;
    let coin = DummyToken::mint(account, amount);
    assert(Token::value<DummyToken>(&coin) == amount, 1);
    assert(Token::market_cap<DummyToken>() == old_market_cap + amount, 2);
    Account::deposit_to_self(account, coin);
    assert(Account::balance<DummyToken>(account_address) == amount, 3);
}
}

// check: EXECUTED
