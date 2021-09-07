//! account: alice, 0 0x1::STC::STC

//! new-transaction
//! sender: alice
script {
use 0x1::STC::{STC};
use 0x1::Token;
use 0x1::Account;
fun main(account: signer) {
    let coin = Account::withdraw<STC>(&account, 0);
    Token::destroy_zero(coin);
}
}
// check: EXECUTED