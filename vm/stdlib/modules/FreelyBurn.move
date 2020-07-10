address 0x1 {
/// Users of any token who plug in this module can burn his coin freely.
module FreelyBurn {
    use 0x1::Token;
    use 0x1::Signer;

    resource struct SharedBurnCapability<TokenType: resource> {
        burn_cap: Token::BurnCapability<TokenType>,
    }

    /// Token Issuer should call this method to plug in the feature
    /// when initializing Token.
    public fun plug_in<TokenType: resource>(signer: &signer, t: &TokenType) {
        let burn_cap = Token::remove_burn_capability<TokenType>(t, Signer::address_of(signer));
        move_to(signer, SharedBurnCapability { burn_cap });
    }

    /// Any one can call this method to burn some tokens.
    public fun burn<TokenType: resource>(token_address: address, tokens: Token::Coin<TokenType>)
    acquires SharedBurnCapability {
        let cap = borrow_global<SharedBurnCapability<TokenType>>(token_address);
        Token::burn_with_capability<TokenType>(&cap.burn_cap, token_address, tokens);
    }
}
}