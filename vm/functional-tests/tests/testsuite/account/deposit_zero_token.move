//! account: alice, 0 0x1::STC::STC

//! new-transaction
//! sender: alice
script {
use 0x1::STC::{STC};
use 0x1::Token;
use 0x1::Account;
fun main(account: signer) {
    let coin = Token::zero<STC>();
    Account::deposit_to_self<STC>(&account, coin);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::STC::{STC};
    use 0x1::Token;
    use 0x1::Account;
    use 0x1::Signer;
    fun main(account: signer) {
        let coin = Token::zero<STC>();
        Account::deposit_with_metadata<STC>(Signer::address_of(&account), coin, x"");
}
}
// check: EXECUTED
