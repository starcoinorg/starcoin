script {
use 0x0::Account;

// TODO: remove initial_amount?
fun main<Token>(fresh_address: address, auth_key_prefix: vector<u8>, initial_amount: u64) {
  Account::create_testnet_account<Token>(fresh_address, auth_key_prefix);
  if (initial_amount > 0) Account::deposit(
        fresh_address,
        Account::withdraw_from_sender<Token>(initial_amount)
     );
}
}
