script {
use 0x1::Account;
fun main<Coin>(account: &signer) {
    Account::add_currency<Coin>(account);
}
}
