script {
use 0x0::Account;
fun main<Coin>(account: &signer) {
    Account::add_currency<Coin>(account);
}
}
