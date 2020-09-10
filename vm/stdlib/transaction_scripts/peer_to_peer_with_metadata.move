script {
use 0x1::Account;

fun peer_to_peer_with_metadata<TokenType>(
    account: &signer,
    payee: address,
    payee_public_key: vector<u8>,
    amount: u128,
    metadata: vector<u8>,
) {
    if (!Account::exists_at(payee)) {
        Account::create_account<TokenType>(payee, payee_public_key);
    };
    Account::pay_from_with_metadata<TokenType>(account,payee, amount, metadata)
}
}
