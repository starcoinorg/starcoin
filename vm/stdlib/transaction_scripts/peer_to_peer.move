script {
use 0x1::Account;

fun peer_to_peer<TokenType>(account: &signer, payee: address, payee_public_key: vector<u8>, amount: u128) {
  if (!Account::exists_at(payee)) Account::create_account<TokenType>(payee, payee_public_key);
  Account::pay_from<TokenType>(account, payee, amount)
}
}
