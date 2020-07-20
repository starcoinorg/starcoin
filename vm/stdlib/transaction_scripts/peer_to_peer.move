script {
use 0x1::Account;

fun main<TokenType>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u128) {
  if (!Account::exists_at(payee)) Account::create_account<TokenType>(payee, copy auth_key_prefix);
  Account::pay_from_sender<TokenType>(account, payee, amount)
}
}
