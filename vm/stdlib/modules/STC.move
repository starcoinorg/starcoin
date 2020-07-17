address 0x1{

module STC {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    struct STC { }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == token_address(), 0);

        Token::register_token<STC>(
            account,
            1000000, // scaling_factor = 10^6
            1000,    // fractional_part = 10^3
        );

        // TODO: whether STC should provide burn cap.
        // let burn_cap = Token::remove_burn_capability<STC>(&t, token_address());
        // Token::destroy_burn_capability(burn_cap);
    }

    /// Returns true if `TokenType` is `STC::STC`
    public fun is_stc<TokenType>(): bool {
        Token::is_registered_in<TokenType>(CoreAddresses::GENESIS_ACCOUNT())
    }

    public fun token_address(): address {
        CoreAddresses::GENESIS_ACCOUNT()
    }
}
}