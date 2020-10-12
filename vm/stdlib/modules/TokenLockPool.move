address 0x1 {
module TokenLockPool {
    use 0x1::Token::{Self, Token};
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::STC::STC;
    use 0x1::ErrorCode;
    use 0x1::Math;

    // A global pool for lock token.
    resource struct TokenPool<TokenType> { token: Token<TokenType> }

    // A fixed time lock key which can withdraw locked token until global time > end time
    resource struct FixedTimeLockKey<TokenType> { total: u128, end_time: u64 }

    // A linear time lock key which can withdraw locked token in a peroid by time-based linear release.
    resource struct LinearTimeLockKey<TokenType> { total: u128, taked: u128, start_time: u64, peroid: u64 }

    // The key which to destory is not empty.
    fun EDESTROY_KEY_NOT_EMPTY(): u64 {
        ErrorCode::ECODE_BASE() + 1
    }

    // Timelock is not unlocked yet.
    fun ETIMELOCK_NOT_UNLOCKED(): u64 {
        ErrorCode::ECODE_BASE() + 2
    }

    // Amount too big than locked token's value.
    fun EAMOUNT_TOO_BIG(): u64 {
        ErrorCode::ECODE_BASE() + 3
    }

    public fun initialize(account: &signer) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        let token_pool = TokenPool<STC> { token: Token::zero() };
        move_to(account, token_pool);
        //TODO how to init other token's pool.
    }

    // Create a LinearTimeLock by token and peroid in seconds.
    public fun create_linear_lock<TokenType>(token: Token<TokenType>, peroid: u64): LinearTimeLockKey<TokenType> acquires TokenPool {
        assert(peroid > 0, ErrorCode::EINVALID_ARGUMENT());
        let start_time = Timestamp::now_seconds();
        let total = Token::value(&token);
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        Token::deposit(&mut token_pool.token, token);
        LinearTimeLockKey<TokenType> {
            total,
            taked: 0,
            start_time,
            peroid
        }
    }

    // Create a FixedTimeLock by token and peroid in seconds.
    public fun create_fixed_lock<TokenType>(token: Token<TokenType>, peroid: u64): FixedTimeLockKey<TokenType> acquires TokenPool {
        assert(peroid > 0, ErrorCode::EINVALID_ARGUMENT());
        let now = Timestamp::now_seconds();
        let total = Token::value(&token);
        let end_time = now + peroid;
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        Token::deposit(&mut token_pool.token, token);
        FixedTimeLockKey<TokenType> {
            total,
            end_time,
        }
    }

    // Unlock token with LinearTimeLockKey
    public fun unlock_with_linear_key<TokenType>(key: &mut LinearTimeLockKey<TokenType>): Token<TokenType> acquires TokenPool {
        let amount = unlocked_amount_of_linear_key(key);
        assert(amount > 0, ETIMELOCK_NOT_UNLOCKED());
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        let token = Token::withdraw(&mut token_pool.token, amount);
        key.taked = key.taked + amount;
        token
    }

    // Unlock token with FixedTimeLockKey
    public fun unlock_with_fixed_key<TokenType>(key: FixedTimeLockKey<TokenType>): Token<TokenType>  acquires TokenPool {
        let amount = unlocked_amount_of_fixed_key(&key);
        assert(amount > 0, ETIMELOCK_NOT_UNLOCKED());
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        let token = Token::withdraw(&mut token_pool.token, key.total);
        let FixedTimeLockKey { total: _, end_time: _ } = key;
        token
    }

    // Returns the unlocked amount of the LinearTimeLockKey.
    // It represent how much Token in the TokenLockPool can bean withdraw by the key now.
    public fun unlocked_amount_of_linear_key<TokenType>(key: &LinearTimeLockKey<TokenType>): u128 {
        let now = Timestamp::now_seconds();
        let elapsed_time = now - key.start_time;
        if (elapsed_time >= key.peroid) {
            key.total - key.taked
        }else {
            Math::mul_div(key.total, (elapsed_time as u128), (key.peroid as u128)) - key.taked
        }
    }

    // Returns the unlocked amount of the FixedTimeLockKey.
    public fun unlocked_amount_of_fixed_key<TokenType>(key: &FixedTimeLockKey<TokenType>): u128 {
        let now = Timestamp::now_seconds();
        if (now >= key.end_time) {
            key.total
        }else{
            0
        }
    }

    public fun end_time_of<TokenType>(key: &FixedTimeLockKey<TokenType>): u64 {
        key.end_time
    }

    public fun destroy_empty<TokenType>(key: LinearTimeLockKey<TokenType>) {
        let LinearTimeLockKey<TokenType> { total, taked, start_time: _, peroid: _ } = key;
        assert(total == taked, EDESTROY_KEY_NOT_EMPTY());
    }

}
}