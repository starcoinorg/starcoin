//# init -n dev

//# faucet --addr alice


//# run --signers alice
script {
use Std::STC::{STC};
use Std::Token;
use Std::Account;
fun main(account: signer) {
    let coin = Token::zero<STC>();
    Account::deposit_to_self<STC>(&account, coin);
}
}
// check: EXECUTED


//# run --signers alice
script {
    use Std::STC::{STC};
    use Std::Token;
    use Std::Account;
    use Std::Signer;
    fun main(account: signer) {
        let coin = Token::zero<STC>();
        Account::deposit_with_metadata<STC>(Signer::address_of(&account), coin, x"");
}
}
// check: EXECUTED
