address 0x1 {
module Balance {
    use 0x1::Token;

    resource struct Balance<TokenType: resource> {
        coin: Token::Coin<TokenType>,
    }

    resource struct WithdrawCapability<TokenType: resource> { }

    /// `Signer` calls this method to accept the Coin.
    public fun accept_token<TokenType: resource>(signer: &signer) {
        let zero_coin = Token::zero<TokenType>();
        let b = Balance { coin: zero_coin };
        move_to(signer, b)
    }

    public fun create_withdraw_capability<TokenType: resource>(
        _token: &TokenType,
    ): WithdrawCapability<TokenType> {
        WithdrawCapability<TokenType> {}
    }

    /// Get the balance of `user`
    public fun balance<TokenType: resource>(user: address): u64 acquires Balance {
        let balance_ref = borrow_global<Balance<TokenType>>(user);
        Token::value(&balance_ref.coin)
    }

    /*     /// Transfer `amount` of Coin from `signer` to `receiver`.
    public fun transfer_to<TokenType: resource>(signer: &signer, receiver: address, amount: u64)
    acquires Balance {
        let withdrawed_token = withdraw<TokenType>(signer, amount);
        deposit_to(receiver, withdrawed_token);
    }
    */

    public fun withdraw_with_capability<TokenType: resource>(
        _cap: &WithdrawCapability<TokenType>,
        from: address,
        amount: u64,
    ): Token::Coin<TokenType> acquires Balance {
        withdraw_from<TokenType>(from, amount)
    }

    public fun deposit_to<TokenType: resource>(receiver: address, coin: Token::Coin<TokenType>)
    acquires Balance {
        let receiver_balance = borrow_global_mut<Balance<TokenType>>(receiver);
        Token::deposit(&mut receiver_balance.coin, coin);
    }

    fun withdraw_from<TokenType: resource>(from: address, amount: u64): Token::Coin<TokenType>
    acquires Balance {
        let balance = borrow_global_mut<Balance<TokenType>>(from);
        Token::withdraw(&mut balance.coin, amount)
    }
}
}