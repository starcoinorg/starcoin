script {
use 0x1::Account;
fun main<Token>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  if (!Account::exists_at(payee)) Account::create_account<Token>(payee, auth_key_prefix);
  Account::mint_to_address<Token>(account, payee, amount);
}
}
