// Test user-defined token
//! account: alice
//! account: bob

//! sender: alice
module MyToken {
    use 0x1::Coin;
    use 0x1::Signer;
    use 0x1::FixedPoint32;

    struct MyToken { }

    public fun init(account: &signer) {
        assert(Signer::address_of(account) == {{alice}}, 8000);

        Coin::register_currency<MyToken>(
                    account,
                    FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
                    1000000, // scaling_factor = 10^6
                    1000,    // fractional_part = 10^3
        );
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::MyToken::{Self, MyToken};
use 0x1::Account;
use 0x1::Coin;
use 0x1::CoreAddresses;
use 0x1::RegisteredCurrencies;

fun main(account: &signer) {
    MyToken::init(account);

    let market_cap = Coin::market_cap<MyToken>();
    assert(market_cap == 0, 8001);

    // STC code is stored in CurrencyRecords[1], CurrencyRecords[0] stores the list head
    assert(RegisteredCurrencies::get_currency_for(CoreAddresses::GENESIS_ACCOUNT(), 1) == b"STC", 8005);
    // MyToken
    assert(RegisteredCurrencies::get_currency_for({{alice}}, 0) == b"MyToken", 8006);

    // Create 'Balance<Token>' resource under sender account, and init with zero
    Account::add_currency<MyToken>(account);
}
}

// check: EXECUTED


// issuer mint
//! new-transaction
//! sender: alice
script {
use 0x1::Account;
use 0x1::Coin;
use {{alice}}::MyToken::{MyToken};
fun main(account: &signer) {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Coin::market_cap<MyToken>();
    let coin = Coin::mint<MyToken>(account, 10000);
    assert(Coin::value<MyToken>(&coin) == 10000, 8002);
    assert(Coin::market_cap<MyToken>() == old_market_cap + 10000, 8003);
    Account::deposit_to_sender<MyToken>(account, coin)
}
}

// check: EXECUTED

// user query
//! new-transaction
//! sender: bob
script {
use 0x1::Coin;
use {{alice}}::MyToken::{MyToken};
fun main() {
    // mint 100 coins and check that the market cap increases appropriately
    let market_cap = Coin::market_cap<MyToken>();
    assert(market_cap == 10000, 8004);
}
}

// check: EXECUTED