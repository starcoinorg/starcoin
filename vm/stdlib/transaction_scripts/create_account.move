script {
use 0x1::Account;

fun create_account<TokenType>(account: &signer, fresh_address: address, public_key_vec: vector<u8>, initial_amount: u128) {
    Account::create_account<TokenType>(fresh_address, public_key_vec);
  if (initial_amount > 0) {
    Account::pay_from<TokenType>(account, fresh_address, initial_amount);
  };
}
}
