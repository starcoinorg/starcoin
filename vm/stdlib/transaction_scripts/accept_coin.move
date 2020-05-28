script {
use 0x0::Account;
fun main<Coin>() {
    Account::add_currency<Coin>();
}
}
