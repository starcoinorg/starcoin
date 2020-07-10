//! account: bob, 10000STC
//! account: alice, 10000STC

//! new-transaction
//! sender: bob
script {
use 0x1::Account;
use 0x1::Coin;
use 0x1::STC::{STC};
use 0x1::TransactionFee;
fun pay_fees(account: &signer) {
    let coin = Account::withdraw_from_sender<STC>(account, 200);
    assert(Coin::value<STC>(&coin) == 200, 8001);
    TransactionFee::pay_fee<STC>(coin);
 }
}
// check: EXECUTED

//! new-transaction
//! sender: genesis
script {
use 0x1::Account;
use 0x1::Coin;
use 0x1::STC::{STC};
use 0x1::TransactionFee;
fun distribute_fees(account: &signer) {
    let coin = TransactionFee::distribute_transaction_fees<STC>(account);
    let value = Coin::value<STC>(&coin);
    assert( value == 200, value);
    Account::deposit_to_sender(account, coin);
}
}
// check: EXECUTED
