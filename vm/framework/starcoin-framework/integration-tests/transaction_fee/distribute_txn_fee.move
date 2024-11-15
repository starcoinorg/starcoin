//# init -n dev

//# faucet --addr Genesis

//# faucet --addr alice

//# faucet --addr bob

//# run --signers bob
script {
use starcoin_framework::account;
use starcoin_framework::Token;
use starcoin_framework::starcoin_coin::{STC};
use starcoin_framework::TransactionFee;
fun pay_fees(account: signer) {
    let coin = coin::withdraw<STC>(&account, 200);
    assert!(Token::value<STC>(&coin) == 200, 8001);
    TransactionFee::pay_fee<STC>(coin);
 }
}


//# run --signers Genesis
script {
use starcoin_framework::account;
use starcoin_framework::Token;
use starcoin_framework::starcoin_coin::{STC};
use starcoin_framework::TransactionFee;
fun distribute_fees(account: signer) {
    let coin = TransactionFee::distribute_transaction_fees<STC>(&account);
    let value = Token::value<STC>(&coin);
    assert!( value >= 200, 10000);
    coin::deposit(&account, coin);
}
}
// check: EXECUTED



//# run --signers alice
script {
use starcoin_framework::account;
use starcoin_framework::starcoin_coin::{STC};
use starcoin_framework::TransactionFee;

fun main(account: signer) {
   let coin = TransactionFee::distribute_transaction_fees<STC>(&account);
   coin::deposit(&account, coin);
}
}

// check: ABORTED

