module starcoin_framework::transfer_scripts {
    use std::vector;
    use starcoin_framework::starcoin_account;


    const EADDRESS_AND_AUTH_KEY_MISMATCH: u64 = 101;
    const ELENGTH_MISMATCH: u64 = 102;
    const EDEPRECATED_FUNCTION: u64 = 19;

    public entry fun peer_to_peer<TokenType>(
        account: &signer,
        payee: address,
        _payee_auth_key: vector<u8>,
        amount: u128
    ) {
        peer_to_peer_v2<TokenType>(account, payee, amount)
    }

    public entry fun peer_to_peer_v2<TokenType>(account: &signer, payee: address, amount: u128) {
        starcoin_account::transfer_coins<TokenType>(account, payee, (amount as u64));
    }

    /// Batch transfer token to others.
    public entry fun batch_peer_to_peer<TokenType: store>(
        account: &signer,
        payeees: vector<address>,
        _payee_auth_keys: vector<vector<u8>>,
        amounts: vector<u128>
    ) {
        batch_peer_to_peer_v2<TokenType>(account, payeees, amounts)
    }

    /// Batch transfer token to others.
    public entry fun batch_peer_to_peer_v2<TokenType>(
        account: &signer,
        payeees: vector<address>,
        amounts: vector<u128>
    ) {
        let amounts_u64 = vector::empty<u64>();
        for (i in 0 .. vector::length(&amounts)) {
            vector::push_back(&mut amounts_u64, (*vector::borrow(&amounts, i) as u64));
        };
        starcoin_account::batch_transfer_coins<TokenType>(account, payeees, amounts_u64);
    }
}