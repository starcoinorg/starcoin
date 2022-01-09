address StarcoinFramework {

module TransferScripts {
    use StarcoinFramework::Account;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Vector;
    const EADDRESS_AND_AUTH_KEY_MISMATCH: u64 = 101;
    const ELENGTH_MISMATCH: u64 = 102;
    const EDEPRECATED_FUNCTION: u64 = 19;

    public(script) fun peer_to_peer<TokenType: store>(account: signer, payee: address, _payee_auth_key: vector<u8>, amount: u128) {
         peer_to_peer_v2<TokenType>(account, payee, amount)
    }

    public(script) fun peer_to_peer_v2<TokenType: store>(account: signer, payee: address, amount: u128) {
        if (!Account::exists_at(payee)) {
            Account::create_account_with_address<TokenType>(payee);
        };
        Account::pay_from<TokenType>(&account, payee, amount)
    }

    /// Batch transfer token to others.
    public(script) fun batch_peer_to_peer<TokenType: store>(account: signer, payeees: vector<address>, _payee_auth_keys: vector<vector<u8>>, amounts: vector<u128>) {
         batch_peer_to_peer_v2<TokenType>(account, payeees, amounts)
    }

    /// Batch transfer token to others.
    public(script) fun batch_peer_to_peer_v2<TokenType: store>(account: signer, payeees: vector<address>, amounts: vector<u128>) {
        let len = Vector::length(&payeees);
        assert!(len == Vector::length(&amounts), ELENGTH_MISMATCH);
        let i = 0;
        while (i < len){
            let payee = *Vector::borrow(&payeees, i);
            if (!Account::exists_at(payee)) {
                Account::create_account_with_address<TokenType>(payee);
            };
            let amount = *Vector::borrow(&amounts, i);
            Account::pay_from<TokenType>(&account, payee, amount);
            i = i + 1;
        }
    }

    public(script) fun peer_to_peer_batch<TokenType: store>(_account: signer, _payeees: vector<u8>, _payee_auth_keys: vector<u8>, _amount: u128) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    public(script) fun peer_to_peer_with_metadata<TokenType: store>(
        account: signer,
        payee: address,
        _payee_auth_key: vector<u8>,
        amount: u128,
        metadata: vector<u8>,
    ) {
         peer_to_peer_with_metadata_v2<TokenType>(account, payee, amount, metadata)
    }

    public(script) fun peer_to_peer_with_metadata_v2<TokenType: store>(
            account: signer,
            payee: address,
            amount: u128,
            metadata: vector<u8>,
    ) {
        if (!Account::exists_at(payee)) {
            Account::create_account_with_address<TokenType>(payee);
        };
        Account::pay_from_with_metadata<TokenType>(&account,payee, amount, metadata)
    }
}
}