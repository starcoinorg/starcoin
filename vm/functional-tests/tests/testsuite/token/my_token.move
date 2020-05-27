// Test user-defined token
//! account: alice
//! account: bob

//! sender: alice
module MyToken {
    use 0x0::Coin;
    use 0x0::Transaction;
    use 0x0::FixedPoint32;

    struct T { }

    public fun init() {
        Transaction::assert(Transaction::sender() == {{alice}}, 8000);

        Coin::register_currency<T>(
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
use {{alice}}::MyToken;
use 0x0::Transaction;
use 0x0::Account;
use 0x0::Coin;
use 0x0::RegisteredCurrencies;
use 0x0::Vector;

fun main() {
    MyToken::init();

    let market_cap = Coin::market_cap<MyToken::T>();
    Transaction::assert(market_cap == 0, 8001);

    let records = RegisteredCurrencies::currency_records();
    // STC + MyToken
    Transaction::assert(Vector::length(&records)==2, 8005);

    let record = Vector::borrow(&records, 1);
    Transaction::assert(RegisteredCurrencies::module_address_of(record) == {{alice}}, 8006);

    // UTF8-encoded "MyToken" as a hex string
    Transaction::assert(RegisteredCurrencies::currency_code_of(record) == x"4d79546f6b656e", 8007);

    // Create 'Balance<Token>' resource under sender account, and init with zero
    Account::add_currency<MyToken::T>();
}
}

// check: EXECUTED


// issuer mint
//! new-transaction
//! sender: alice
script {
use 0x0::Transaction;
use 0x0::Account;
use 0x0::Coin;
use {{alice}}::MyToken;
fun main() {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Coin::market_cap<MyToken::T>();
    let coin = Coin::mint<MyToken::T>(10000);
    Transaction::assert(Coin::value<MyToken::T>(&coin) == 10000, 8002);
    Transaction::assert(Coin::market_cap<MyToken::T>() == old_market_cap + 10000, 8003);
    Account::deposit_to_sender<MyToken::T>(coin)
}
}

// check: EXECUTED

// user query
//! new-transaction
//! sender: bob
script {
use 0x0::Transaction;
use 0x0::Coin;
use {{alice}}::MyToken;
fun main() {
    // mint 100 coins and check that the market cap increases appropriately
    let market_cap = Coin::market_cap<MyToken::T>();
    Transaction::assert(market_cap == 10000, 8004);
}
}

// check: EXECUTED