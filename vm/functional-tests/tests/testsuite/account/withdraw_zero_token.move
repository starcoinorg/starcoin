//# init -n dev

//# faucet --addr alice


//# run --signers alice
script {
use Std::STC::{STC};
use Std::Token;
use Std::Account;
fun main(account: signer) {
    let coin = Account::withdraw<STC>(&account, 0);
    Token::destroy_zero(coin);
}
}
// check: EXECUTED