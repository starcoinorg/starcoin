script {
use 0x0::Account;
fun main<Currency>() {
    Account::add_currency<Currency>();
}
}
