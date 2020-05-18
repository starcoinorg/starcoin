// Test user-defined token
//! account: alice
//! account: bob

//! sender: alice
module MyToken {
    use 0x0::Coin;
    //use 0x0::Account;
    use 0x0::Transaction;
    use 0x0::FixedPoint32;

    struct T { }

    public fun init() {
        Transaction::assert(Transaction::sender() == {{alice}}, 8000);

        Coin::register_currency<T>(
                    FixedPoint32::create_from_rational(1, 1), // exchange rate to LBR
                    true,    // is_synthetic
                    1000000, // scaling_factor = 10^6
                    1000,    // fractional_part = 10^3
                    x"4d79546f6b656e" // UTF8-encoded "MyToken" as a hex string
        );

        // mint 100 coins and check that the market cap increases appropriately
        //let old_market_cap = Coin::market_cap<T>();
        //let coin = Coin::mint<T>(10000);
        //Transaction::assert(Coin::value<T>(&coin) == 10000, 8001);
        //Transaction::assert(Coin::market_cap<T>() == old_market_cap + 10000, 8002);

        // Create 'Balance<Token>' resource under sender account
        //Account::add_currency<T>();
        //Account::deposit_to_sender<T>(coin)

    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::MyToken;
fun main() {
    MyToken::init();

}
}

// check: EXECUTED
