script {
use 0x0::Account;

fun main<Token>(
    payee: address,
    auth_key_prefix: vector<u8>,
    amount: u64,
    metadata: vector<u8>,
    metadata_signature: vector<u8>
) {
  if (!Account::exists(payee)) {
      Account::create_testnet_account<Token>(payee, auth_key_prefix);
  };
  Account::pay_from_sender_with_metadata<Token>(payee, amount, metadata, metadata_signature)
}
}
