//! account: alice, 0STC

// Minting from a privileged account should work
//! sender: association
script {
use 0x0::STC;
use 0x0::Coin;
use 0x0::Account;
use 0x0::Transaction;
fun main(account: &signer) {
    let old_market_cap = Coin::market_cap<STC::T>();
    let coin = Coin::mint<STC::T>(account, 100);
    Transaction::assert(Coin::value<STC::T>(&coin) == 100, 8000);
    Transaction::assert(Coin::market_cap<STC::T>() == old_market_cap + 100, 8001);

    // get rid of the coin
    Account::deposit(account, {{alice}}, coin);
}
}

// check: EXECUTED
// check: increment_size
// check: 0

//! new-transaction
// Minting from a non-privileged account should not work
script {
use 0x0::STC;
use 0x0::Coin;
use 0x0::Account;
fun main(account: &signer) {
    let coin = Coin::mint<STC::T>(account, 100);
    Account::deposit_to_sender<STC::T>(account, coin)
}
}

// will fail with MISSING_DATA because sender doesn't have the mint capability
// check: Keep
// check: MISSING_DATA
