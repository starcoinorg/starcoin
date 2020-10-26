address 0x1{

module DummyToken {
    use 0x1::Token::{Self, Token};
    use 0x1::Errors;

    struct DummyToken { }


    const EMINT_TOO_MUCH:u64 = 101;

    const PRECISION: u8 = 3;

    resource struct SharedBurnCapability{
        cap: Token::BurnCapability<DummyToken>,
    }

    resource struct SharedMintCapability{
        cap: Token::MintCapability<DummyToken>,
    }

    public fun initialize(account: &signer) {
        Token::register_token<DummyToken>(
            account,
            PRECISION,
        );

        let burn_cap = Token::remove_burn_capability<DummyToken>(account);
        move_to(account, SharedBurnCapability{cap: burn_cap});

        let burn_cap = Token::remove_mint_capability<DummyToken>(account);
        move_to(account, SharedMintCapability{cap: burn_cap});
    }

    /// Returns true if `TokenType` is `DummyToken::DummyToken`
    public fun is_dummy_token<TokenType>(): bool {
        Token::is_same_token<DummyToken, TokenType>()
    }

    public fun burn(token: Token<DummyToken>) acquires SharedBurnCapability{
        let cap = borrow_global<SharedBurnCapability>(token_address());
        Token::burn_with_capability(&cap.cap, token);
    }

    /// Anyone can mint DummyToken, amount should < 10000
    public fun mint(_account: &signer, amount: u128) : Token<DummyToken> acquires SharedMintCapability{
        assert(amount <= 10000, Errors::invalid_argument(EMINT_TOO_MUCH));
        let cap = borrow_global<SharedMintCapability>(token_address());
        Token::mint_with_capability(&cap.cap, amount)
    }

    public fun token_address(): address {
        Token::token_address<DummyToken>()
    }
}
}