script {
use 0x1::Account;

fun main<Token>(
    account: &signer,
    payee: address,
    auth_key_prefix: vector<u8>,
    amount: u64,
    metadata: vector<u8>,
    metadata_signature: vector<u8>
) {
  if (!Account::exists_at(payee)) {
      Account::create_account<Token>(payee, auth_key_prefix);
  };
  Account::pay_from_sender_with_metadata<Token>(account,payee, amount, metadata, metadata_signature)
}
}
