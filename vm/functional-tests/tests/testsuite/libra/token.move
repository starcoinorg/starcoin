// Test user-defined token
//! account: alice
//! account: bob

//! sender: alice
module MyToken {
    use 0x0::Libra;
    use 0x0::LibraAccount;
    use 0x0::Transaction;
    struct T { }

    public fun new() {
        Transaction::assert(Transaction::sender() == {{alice}}, 8000);
        Libra::register<T>();
        // mint 100 coins and check that the market cap increases appropriately
        let old_market_cap = Libra::market_cap<T>();
        let coin = Libra::mint<T>(10000);
        Transaction::assert(Libra::value<T>(&coin) == 10000, 8001);
        Transaction::assert(Libra::market_cap<T>() == old_market_cap + 10000, 8002);

        // Create 'Balance<Token>' resource under sender account
        LibraAccount::create_new_balance<T>();
        LibraAccount::deposit_to_sender<T>(coin)

    }
}

//! new-transaction
//! sender: alice

use {{alice}}::MyToken;
fun main() {
    MyToken::new();

}

// check: EXECUTED

//! new-transaction
//! sender: bob

use {{alice}}::MyToken;
use 0x0::LibraAccount;

fun main() {
    // Create 'Balance<Token>' resource under sender account to receive token
    LibraAccount::create_new_balance<MyToken::T>();
}

//! new-transaction
//! sender: alice

use {{alice}}::MyToken;
use 0x0::LibraAccount;
use 0x0::Transaction;
use 0x0::Vector;

fun main() {
    LibraAccount::pay_from_sender<MyToken::T>({{bob}}, Vector::empty<u8>(), 10);
    let balance = LibraAccount::balance<MyToken::T>({{bob}});
    Transaction::assert(balance == 10, 8003)
}
