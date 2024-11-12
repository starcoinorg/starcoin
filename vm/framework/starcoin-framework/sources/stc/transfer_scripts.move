module starcoin_framework::transfer_scripts {

    use std::vector;
    use starcoin_framework::create_signer;
    use starcoin_std::debug;
    use starcoin_framework::coin;
    use starcoin_framework::account;

    const EADDRESS_AND_AUTH_KEY_MISMATCH: u64 = 101;
    const ELENGTH_MISMATCH: u64 = 102;
    const EDEPRECATED_FUNCTION: u64 = 19;

    public entry fun peer_to_peer<TokenType: store>(
        account: signer,
        payee: address,
        _payee_auth_key: vector<u8>,
        amount: u128
    ) {
        peer_to_peer_v2<TokenType>(account, payee, amount)
    }

    public entry fun peer_to_peer_v2<TokenType>(account: signer, payee: address, amount: u128) {
        debug::print(&std::string::utf8(b"transfer_scripts::peer_to_peer_v2 | Entered"));
        debug::print(&coin::name<TokenType>());
        debug::print(&coin::symbol<TokenType>());
        account::create_account_if_does_not_exist(payee);
        coin::register<TokenType>(&create_signer::create_signer(payee));
        coin::transfer<TokenType>(&account, payee, (amount as u64));
        debug::print(&std::string::utf8(b"transfer_scripts::peer_to_peer_v2 | Exited"));
    }

    /// Batch transfer token to others.
    public entry fun batch_peer_to_peer<TokenType: store>(
        account: signer,
        payeees: vector<address>,
        _payee_auth_keys: vector<vector<u8>>,
        amounts: vector<u128>
    ) {
        batch_peer_to_peer_v2<TokenType>(account, payeees, amounts)
    }

    /// Batch transfer token to others.
    public entry fun batch_peer_to_peer_v2<TokenType: store>(
        account: signer,
        payeees: vector<address>,
        amounts: vector<u128>
    ) {
        let len = vector::length(&payeees);
        assert!(len == vector::length(&amounts), ELENGTH_MISMATCH);
        let i = 0;
        while (i < len) {
            let payee = *vector::borrow(&payeees, i);
            account::create_account_if_does_not_exist(payee);
            coin::register<TokenType>(&account);
            let amount = *vector::borrow(&amounts, i);
            coin::transfer<TokenType>(&account, payee, (amount as u64));
            i = i + 1;
        }
    }

}