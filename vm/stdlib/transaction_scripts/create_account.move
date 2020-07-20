script {
use 0x1::Account;

// TODO: remove initial_amount?
fun main<TokenType>(account: &signer, fresh_address: address, auth_key_prefix: vector<u8>, initial_amount: u64) {
  Account::create_account<TokenType>(fresh_address, auth_key_prefix);
  if (initial_amount > 0) Account::deposit(account,
        fresh_address,
        Account::withdraw_from_sender<TokenType>(account, initial_amount)
     );
}
}
