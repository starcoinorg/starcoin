address StarcoinFramework{
/// The module provide a dummy token implementation.
module DummyToken {
    use StarcoinFramework::Token::{Self, Token};
    use StarcoinFramework::Errors;

    /// The DummyToken type.
    struct DummyToken has copy, drop, store { }


    const EMINT_TOO_MUCH:u64 = 101;

    const PRECISION: u8 = 3;

    /// Burn capability of the token.
    struct SharedBurnCapability has key {
        cap: Token::BurnCapability<DummyToken>,
    }

    /// Mint capability of the token.
    struct SharedMintCapability has key, store {
        cap: Token::MintCapability<DummyToken>,
    }

    /// Initialization of the module.
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
    public fun is_dummy_token<TokenType: store>(): bool {
        Token::is_same_token<DummyToken, TokenType>()
    }

    /// Burn the given token.
    public fun burn(token: Token<DummyToken>) acquires SharedBurnCapability{
        let cap = borrow_global<SharedBurnCapability>(token_address());
        Token::burn_with_capability(&cap.cap, token);
    }

    /// Anyone can mint DummyToken, amount should < 10000
    public fun mint(_account: &signer, amount: u128) : Token<DummyToken> acquires SharedMintCapability{
        assert!(amount <= 10000, Errors::invalid_argument(EMINT_TOO_MUCH));
        let cap = borrow_global<SharedMintCapability>(token_address());
        Token::mint_with_capability(&cap.cap, amount)
    }

    /// Return the token address.
    public fun token_address(): address {
        Token::token_address<DummyToken>()
    }
}

module DummyTokenScripts{
    use StarcoinFramework::DummyToken::{Self,DummyToken};
    use StarcoinFramework::Account;
    use StarcoinFramework::Signer;

    public(script) fun mint(sender: signer, amount: u128){
        let token = DummyToken::mint(&sender, amount);
        let sender_addr = Signer::address_of(&sender);
        if(Account::is_accept_token<DummyToken>(sender_addr)){
            Account::do_accept_token<DummyToken>(&sender);
        };
        Account::deposit(sender_addr, token);
    }
}
}