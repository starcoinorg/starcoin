address 0x1 {
/**
    Fixed Quantity Token Example.
    Token Issuer register the token, and mint the specified token to himself.
    After this, no one can mint again, even token issuer himself.
    Toekn issuer can dispatch the minted token to others.
*/
module FixedQuantityCoin {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Balance;
    use 0x1::TrivalTransfer;

    /// CoinType of FixedQuantityCoin
    resource struct T { }

    // const TOKEN_ADDRESS: address = 0x1;
    /// Total Supply: 100 million.
    // const TOTAL_SUPPLY: u64 = 100000000;

    /// Initialize method of the FixedQuantityCoin.
    /// should only be called once by the Token Issuer.
    public fun initialize(signer: &signer) {
        assert(Signer::address_of(signer) == token_address(), 401);
        let t = T {};
        // register currency.
        Token::register_currency<T>(signer, &t, 1000, 1000);
        Balance::accept_token<T>(signer);
        TrivalTransfer::plug_in<T>(signer, &t);
        // Mint all to myself at the beginning.
        let minted_token = Token::mint<T>(signer, total_supply(), token_address());
        Balance::deposit_to(Signer::address_of(signer), minted_token);
        // destroy mint cap from myself.
        let mint_cap = Token::remove_my_mint_capability<T>(signer);
        Token::destroy_mint_capability(mint_cap);
        // destroy T, so that no one can mint.
        let T{  } = t;
    }

    /// `Signer` calls this method to accept the Coin.
    public fun accept(signer: &signer) {
        Balance::accept_token<T>(signer);
    }

    /// Get the balance of `user`
    public fun balance(user: address): u64 {
        Balance::balance<T>(user)
    }

    /// Transfer `amount` of Coin from `signer` to `receiver`.
    public fun transfer_to(signer: &signer, receiver: address, amount: u64) {
        TrivalTransfer::transfer<T>(signer, token_address(), receiver, amount);
    }

    public fun token_address(): address {
        0x1
    }

    public fun total_supply(): u64 {
        100000000
    }
}
}