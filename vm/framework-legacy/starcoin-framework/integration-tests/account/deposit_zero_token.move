//# init -n dev

//# faucet --addr alice


//# run --signers alice
script {
use StarcoinFramework::STC::{STC};
use StarcoinFramework::Token;
use StarcoinFramework::Account;
fun main(account: signer) {
    let coin = Token::zero<STC>();
    Account::deposit_to_self<STC>(&account, coin);
}
}
// check: EXECUTED


//# run --signers alice
script {
    use StarcoinFramework::STC::{STC};
    use StarcoinFramework::Token;
    use StarcoinFramework::Account;
    use StarcoinFramework::Signer;
    fun main(account: signer) {
        let coin = Token::zero<STC>();
        Account::deposit_with_metadata<STC>(Signer::address_of(&account), coin, x"");
}
}
// check: EXECUTED
