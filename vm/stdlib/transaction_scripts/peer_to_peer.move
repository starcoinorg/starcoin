script {
use 0x0::Account;

fun main<Token>(account: &signer, payee: address, auth_key_prefix: vector<u8>, amount: u64) {
  if (!Account::exists(payee)) Account::create_account<Token>(payee, copy auth_key_prefix);
  Account::pay_from_sender<Token>(account, payee, amount)
}
}
