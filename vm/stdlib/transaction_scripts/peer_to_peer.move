script {
use 0x1::Account;

fun peer_to_peer<TokenType>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u128) {
  if (!Account::exists_at(payee)) Account::create_account<TokenType>(payee, copy auth_key_prefix);
  Account::pay_from<TokenType>(account, payee, amount)
}
}
