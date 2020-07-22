script {
use 0x1::Account;
fun main<TokenType>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u128) {
  if (!Account::exists_at(payee)) Account::create_account<TokenType>(payee, auth_key_prefix);
  Account::mint_to_address<TokenType>(account, payee, amount);
}
}
