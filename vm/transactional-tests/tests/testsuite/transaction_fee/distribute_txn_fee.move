//# init -n dev

//# faucet --addr Genesis

//# faucet --addr alice

//# faucet --addr bob

//# run --signers bob
script {
use Std::Account;
use Std::Token;
use Std::STC::{STC};
use Std::TransactionFee;
fun pay_fees(account: signer) {
    let coin = Account::withdraw<STC>(&account, 200);
    assert!(Token::value<STC>(&coin) == 200, 8001);
    TransactionFee::pay_fee<STC>(coin);
 }
}


//# run --signers Genesis
script {
use Std::Account;
use Std::Token;
use Std::STC::{STC};
use Std::TransactionFee;
fun distribute_fees(account: signer) {
    let coin = TransactionFee::distribute_transaction_fees<STC>(&account);
    let value = Token::value<STC>(&coin);
    assert!( value >= 200, 10000);
    Account::deposit_to_self(&account, coin);
}
}
// check: EXECUTED



//# run --signers alice
script {
use Std::Account;
use Std::STC::{STC};
use Std::TransactionFee;

fun main(account: signer) {
   let coin = TransactionFee::distribute_transaction_fees<STC>(&account);
   Account::deposit_to_self(&account, coin);
}
}

// check: ABORTED

