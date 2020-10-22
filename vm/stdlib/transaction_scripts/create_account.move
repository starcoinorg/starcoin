script {
use 0x1::Account;
use 0x1::Errors;
const EADDRESS_AND_AUTH_KEY_MISMATCH: u64 = 101;
fun create_account<TokenType>(account: &signer, fresh_address: address, auth_key: vector<u8>, initial_amount: u128) {
  let created_address = Account::create_account<TokenType>(auth_key);
  assert(fresh_address == created_address, Errors::invalid_argument(EADDRESS_AND_AUTH_KEY_MISMATCH));
  if (initial_amount > 0) {
    Account::pay_from<TokenType>(account, fresh_address, initial_amount);
  };
}
}
