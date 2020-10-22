script {
use 0x1::Account;
use 0x1::Errors;
const EADDRESS_AND_AUTH_KEY_MISMATCH: u64 = 101;
fun peer_to_peer_with_metadata<TokenType>(
    account: &signer,
    payee: address,
    payee_auth_key: vector<u8>,
    amount: u128,
    metadata: vector<u8>,
) {
    if (!Account::exists_at(payee)) {
        let created_address = Account::create_account<TokenType>(payee_auth_key);
        assert(payee == created_address, Errors::invalid_argument(EADDRESS_AND_AUTH_KEY_MISMATCH));
    };
    Account::pay_from_with_metadata<TokenType>(account,payee, amount, metadata)
}
}
