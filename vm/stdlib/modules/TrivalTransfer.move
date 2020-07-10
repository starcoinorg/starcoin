address 0x1 {
/// A convenient module for Token who just want to have a trival transfer functionality./// Token issuer plugs in the module, and `transfer` works.
module TrivalTransfer {
    use 0x1::Balance;
    use 0x1::Signer;
    use 0x1::Token;

    resource struct SharedCapability<TokenType: resource> {
        withdraw_cap: Balance::WithdrawCapability<TokenType>,
    }

    /// Users of token who plug_in this module has the ability to transfer coin freely.
    public fun plug_in<TokenType: resource>(signer: &signer, t: &TokenType) {
        // create shared withdraw capability
        let withdraw_cap = Balance::create_withdraw_capability<TokenType>(t);
        move_to(signer, SharedCapability { withdraw_cap });
    }

    /// Anyone can call this method to withdraw some tokens from himself.
    public fun withdraw<TokenType: resource>(
        signer: &signer,
        token_address: address,
        amount: u64,
    ): Token::Coin<TokenType> acquires SharedCapability {
        let shared_cap = borrow_global<SharedCapability<TokenType>>(token_address);
        Balance::withdraw_with_capability<TokenType>(
            &shared_cap.withdraw_cap,
            Signer::address_of(signer),
            amount,
        )
    }

    /// Anyone call call this method to transfer some tokens from himself to others.
    public fun transfer<TokenType: resource>(
        signer: &signer,
        token_address: address,
        receiver: address,
        amount: u64,
    ) acquires SharedCapability {
        let coins = withdraw<TokenType>(signer, token_address, amount);
        Balance::deposit_to<TokenType>(receiver, coins);
    }
}
}