address 0x1 {
module TokenBox {
    use 0x1::Token::{Self, Token};
    use 0x1::Timestamp;
    use 0x1::Signer;

    // A wrapper around Token, and add a timelock, the token can bean withdraw in a peroid by time-based linear release.
    resource struct TokenBox<TokenType> { origin: u128, token: Token<TokenType>, lock_time: u64, lock_peroid: u64 }

    // Create a TokenBox by token and lock_peroid in seconds.
    public fun create<TokenType>(token: Token<TokenType>, lock_peroid: u64): TokenBox<TokenType> {
        let lock_time = Timestamp::now_seconds();
        let origin = Token::share(&token);
        TokenBox<TokenType> {
            origin,
            token,
            lock_time,
            lock_peroid
        }
    }

    // Withdraw unlocked token in the TokenBox.
    public fun withdraw<TokenType>(token_box: &mut TokenBox<TokenType>): Token<TokenType> {
        let value = unlocked_value_of(token_box);
        Token::withdraw_share(&mut token_box.token, value)
    }

    // Split oen token box to two box.
    public fun split<TokenType>(token_box: TokenBox<TokenType>, amount: u128): (TokenBox<TokenType>, TokenBox<TokenType>) {
        let TokenBox<TokenType> { origin, token, lock_time, lock_peroid } = token_box;
        let (t1, t2) = Token::split_share(token, amount);
        (TokenBox<TokenType> { origin, token: t1, lock_time, lock_peroid }, TokenBox<TokenType> { origin, token: t2, lock_time, lock_peroid })
    }

    // Returns the unlocked value of the TokenBox.
    // It represent how much Token in the TokenBox can bean withdraw now.
    public fun unlocked_value_of<TokenType>(token_box: &TokenBox<TokenType>): u128 {
        let now = Timestamp::now_seconds();
        let elapsed_time = now - token_box.lock_time;
        if (elapsed_time >= token_box.lock_peroid) {
            return Token::share(&token_box.token)
        }else {
            token_box.origin * (elapsed_time as u128) / (token_box.lock_peroid as u128)
        }
    }

    public fun destroy_empty<TokenType>(token_box: TokenBox<TokenType>) {
        let TokenBox<TokenType> { origin: _, token, lock_time: _, lock_peroid: _ } = token_box;
        Token::destroy_zero(token);
    }

    // Save TokenBox to sender account.
    public fun save<TokenType>(account: &signer, token_box: TokenBox<TokenType>) {
        move_to(account, token_box);
    }

    // Take TokenBox from sender account.
    public fun take<TokenType>(account: &signer): TokenBox<TokenType> acquires TokenBox {
        move_from<TokenBox<TokenType>>(Signer::address_of(account))
    }

    public fun exists_at<TokenType>(address: address): bool {
        exists<TokenBox<TokenType>>(address)
    }
}
}