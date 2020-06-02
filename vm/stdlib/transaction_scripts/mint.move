script {
use 0x0::Account;
fun main<Token>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  if (!Account::exists(payee)) Account::create_testnet_account<Token>(payee, auth_key_prefix);
  Account::mint_to_address<Token>(account, payee, amount);
}
}
