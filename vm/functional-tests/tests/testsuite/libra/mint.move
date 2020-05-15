// Test the mint flow

//! account: alice, 0STC

// Minting from a privileged account should work
//! sender: association
script {
use 0x0::STC;
use 0x0::Libra;
use 0x0::LibraAccount;
use 0x0::Transaction;
fun main() {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Libra::market_cap<STC::T>();
    let coin = Libra::mint<STC::T>(100);
    Transaction::assert(Libra::value<STC::T>(&coin) == 100, 8000);
    Transaction::assert(Libra::market_cap<STC::T>() == old_market_cap + 100, 8001);

    // get rid of the coin
    LibraAccount::deposit({{alice}}, coin);
}
}

// check: EXECUTED

//! new-transaction
// Minting from a non-privileged account should not work
script {
use 0x0::STC;
use 0x0::Libra;
use 0x0::LibraAccount;
fun main() {
    let coin = Libra::mint<STC::T>(100);
    LibraAccount::deposit_to_sender<STC::T>(coin)
}
}

// will fail with MISSING_DATA because sender doesn't have the mint capability
// check: Keep
// check: MISSING_DATA
