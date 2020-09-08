script {
use 0x1::Account;

fun peer_to_peer_with_metadata<TokenType>(
    account: &signer,
    payee_public_key: vector<u8>,
    amount: u128,
    metadata: vector<u8>,
) {
    let payee = Account::create_account<TokenType>(copy payee_public_key);
    Account::pay_from_with_metadata<TokenType>(account,payee, amount, metadata)
}
}
