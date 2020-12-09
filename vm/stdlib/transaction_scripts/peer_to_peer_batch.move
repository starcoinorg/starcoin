script {
    use 0x1::Account;
    use 0x1::Errors;
    use 0x1::Vector;
    use 0x1::SCS;
    const EADDRESS_AND_AUTH_KEY_MISMATCH: u64 = 101;
    fun peer_to_peer_batch<TokenType>(account: &signer, payeees: vector<u8>, payee_auth_keys: vector<u8>, amount: u128) {
        let payee_bytes_vec = Vector::split<u8>(&payeees, 16);
        let auth_key_bytes_vec = Vector::split<u8>(&payee_auth_keys, 32);
        let len = Vector::length(&payee_bytes_vec);
        let i = 0;
        while (i < len){
            let payee_bytes  = *Vector::borrow<vector<u8>>(&payee_bytes_vec, i);
            let payee = SCS::to_address(payee_bytes);
            let payee_auth_key = *Vector::borrow<vector<u8>>(&auth_key_bytes_vec, i);
            if (!Account::exists_at(payee)) {
            let created_address = Account::create_account<TokenType>(payee_auth_key);
            assert(payee == created_address, Errors::invalid_argument(EADDRESS_AND_AUTH_KEY_MISMATCH));
            };
            Account::pay_from<TokenType>(account, payee, amount);
            i = i + 1;
        }
    }
}
