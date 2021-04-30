address 0x1 {

module TransferScripts {
    use 0x1::Account;
    use 0x1::Errors;
    use 0x1::Vector;
    use 0x1::BCS;
    const EADDRESS_AND_AUTH_KEY_MISMATCH: u64 = 101;
    const ELENGTH_MISMATCH: u64 = 102;
    public(script) fun peer_to_peer<TokenType: store>(account: signer, payee: address, payee_auth_key: vector<u8>, amount: u128) {
        if (!Account::exists_at(payee)) {
            let created_address = Account::create_account<TokenType>(payee_auth_key);
            assert(payee == created_address, Errors::invalid_argument(EADDRESS_AND_AUTH_KEY_MISMATCH));
        };
        Account::pay_from<TokenType>(&account, payee, amount)
    }

    /// Batch transfer token to others.
    public(script) fun batch_peer_to_peer<TokenType: store>(account: signer, payeees: vector<address>, payee_auth_keys: vector<vector<u8>>, amounts: vector<u128>) {
        let len = Vector::length(&payeees);
        assert(len == Vector::length(&payee_auth_keys), ELENGTH_MISMATCH);
        assert(len == Vector::length(&amounts), ELENGTH_MISMATCH);
        let i = 0;
        while (i < len){
            let payee = *Vector::borrow(&payeees, i);
            let payee_auth_key = *Vector::borrow(&payee_auth_keys, i);
            if (!Account::exists_at(payee)) {
                let created_address = Account::create_account<TokenType>(payee_auth_key);
                assert(payee == created_address, Errors::invalid_argument(EADDRESS_AND_AUTH_KEY_MISMATCH));
            };
            let amount = *Vector::borrow(&amounts, i);
            Account::pay_from<TokenType>(&account, payee, amount);
            i = i + 1;
        }
    }

    public(script) fun peer_to_peer_batch<TokenType: store>(account: signer, payeees: vector<u8>, payee_auth_keys: vector<u8>, amount: u128) {
        let payee_bytes_vec = Vector::split<u8>(&payeees, 16);
        let auth_key_bytes_vec = Vector::split<u8>(&payee_auth_keys, 32);
        let len = Vector::length(&payee_bytes_vec);
        let i = 0;
        while (i < len){
            let payee_bytes  = *Vector::borrow<vector<u8>>(&payee_bytes_vec, i);
            let payee = BCS::to_address(payee_bytes);
            let payee_auth_key = *Vector::borrow<vector<u8>>(&auth_key_bytes_vec, i);
            if (!Account::exists_at(payee)) {
                let created_address = Account::create_account<TokenType>(payee_auth_key);
                assert(payee == created_address, Errors::invalid_argument(EADDRESS_AND_AUTH_KEY_MISMATCH));
            };
            Account::pay_from<TokenType>(&account, payee, amount);
            i = i + 1;
        }
    }

    public(script) fun peer_to_peer_with_metadata<TokenType: store>(
        account: signer,
        payee: address,
        payee_auth_key: vector<u8>,
        amount: u128,
        metadata: vector<u8>,
    ) {
        if (!Account::exists_at(payee)) {
            let created_address = Account::create_account<TokenType>(payee_auth_key);
            assert(payee == created_address, Errors::invalid_argument(EADDRESS_AND_AUTH_KEY_MISMATCH));
        };
        Account::pay_from_with_metadata<TokenType>(&account,payee, amount, metadata)
    }
}
}