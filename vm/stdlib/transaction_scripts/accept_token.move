script {
use 0x1::Account;
fun main<TokenType>(account: &signer) {
    Account::accept_token<TokenType>(account);
}
}
