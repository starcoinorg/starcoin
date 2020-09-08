script {
use 0x1::Account;

fun peer_to_peer<TokenType>(account: &signer, payee_public_key: vector<u8>, amount: u128) {
  let new_address = Account::create_account<TokenType>(copy payee_public_key);
  Account::pay_from<TokenType>(account, new_address, amount)
}
}
