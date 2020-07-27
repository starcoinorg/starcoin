address 0x1{

module STC {
    use 0x1::Token::{Self, Token};
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    struct STC { }

    const SCALING_FACTOR : u128 = 1000000;
    const FRACTIONAL_PART: u128 = 1000;
    resource struct SharedBurnCapability{
        cap: Token::BurnCapability<STC>,
    }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == token_address(), 0);

        Token::register_token<STC>(
            account,
            SCALING_FACTOR, // scaling_factor = 10^6
            FRACTIONAL_PART,    // fractional_part = 10^3
        );

        let burn_cap = Token::remove_burn_capability<STC>(account);
        move_to(account, SharedBurnCapability{cap: burn_cap});
    }

    /// Returns true if `TokenType` is `STC::STC`
    public fun is_stc<TokenType>(): bool {
        Token::is_registered_in<TokenType>(CoreAddresses::GENESIS_ACCOUNT())
    }

    public fun burn(token: Token<STC>) acquires SharedBurnCapability{
        let cap = borrow_global<SharedBurnCapability>(token_address());
        Token::burn_with_capability(&cap.cap, token);
    }

    public fun token_address(): address {
        CoreAddresses::GENESIS_ACCOUNT()
    }
}
}