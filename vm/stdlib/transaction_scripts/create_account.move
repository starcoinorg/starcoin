script {
use 0x1::Account;

fun create_account<TokenType>(account: &signer, public_key_vec: vector<u8>, initial_amount: u128) {
    let new_address = Account::create_account<TokenType>(public_key_vec);
  if (initial_amount > 0) Account::deposit_to(account,
        new_address,
        Account::withdraw<TokenType>(account, initial_amount)
     );
}
}
