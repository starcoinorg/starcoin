script {
use 0x1::Account;
fun accept_token<TokenType: store>(account: &signer) {
    Account::accept_token<TokenType>(account);
}
}
