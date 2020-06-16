// Test the gas check flow

//! account: alice, 0STC

//! sender: association
script {
use 0x0::STC;
use 0x0::Coin;
use 0x0::Account;
use 0x0::Transaction;
//! gas-price: 1
//! max-gas: 10000
fun main(account: &signer) {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Coin::market_cap<STC::T>();
    let coin = Coin::mint<STC::T>(account, 100);
    Transaction::assert(Coin::value<STC::T>(&coin) == 100, 8000);
    Transaction::assert(Coin::market_cap<STC::T>() == old_market_cap + 100, 8001);

    // get rid of the coin
    Account::deposit(account, {{alice}}, coin);
}
// check: gas_used
// check: 10000
// check: OUT_OF_GAS
}

