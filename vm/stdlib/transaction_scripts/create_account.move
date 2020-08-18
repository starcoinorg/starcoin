script {
use 0x1::Account;

fun create_account<TokenType>(account: &signer, fresh_address: address, auth_key_prefix: vector<u8>, initial_amount: u128) {
  Account::create_account<TokenType>(fresh_address, auth_key_prefix);
  if (initial_amount > 0) Account::deposit_to(account,
        fresh_address,
        Account::withdraw<TokenType>(account, initial_amount)
     );
}
}
