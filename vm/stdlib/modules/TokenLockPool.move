address 0x1 {
module TokenLockPool {
    use 0x1::Token::{Self, Token};
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::STC::STC;
    use 0x1::ErrorCode;

    // A global pool for lock token.
    resource struct TokenPool<TokenType> { token: Token<TokenType> }

    // A fixed time lock key which can withdraw locked token until global time > time lock
    resource struct FixedTimeLockKey<TokenType> { origin: u128, time_lock: u64 }

    // A linear time lock key which can withdraw locked token in a peroid by time-based linear release.
    resource struct LinearTimeLockKey<TokenType> { origin: u128, taked: u128, lock_time: u64, lock_peroid: u64 }

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

    // Peroid is zero
    fun EPEROID_IS_ZERO(): u64 {
        ErrorCode::ECODE_BASE() + 4
    }

    public fun initialize(account: &signer) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        let token_pool = TokenPool<STC> { token: Token::zero() };
        move_to(account, token_pool);
        //TODO how to init other token's pool.
    }

    // Create a LinearTimeLock by token and lock_peroid in seconds.
    public fun create_linear_lock<TokenType>(token: Token<TokenType>, lock_peroid: u64): LinearTimeLockKey<TokenType> acquires TokenPool {
        assert(lock_peroid > 0, EPEROID_IS_ZERO());
        let lock_time = Timestamp::now_seconds();
        let origin = Token::share(&token);
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        Token::deposit(&mut token_pool.token, token);
        LinearTimeLockKey<TokenType> {
            origin,
            taked: 0,
            lock_time,
            lock_peroid
        }
    }

    // Create a FixedTimeLock by token and lock_peroid in seconds.
    public fun create_fixed_lock<TokenType>(token: Token<TokenType>, lock_peroid: u64): FixedTimeLockKey<TokenType> acquires TokenPool {
        assert(lock_peroid > 0, EPEROID_IS_ZERO());
        let now = Timestamp::now_seconds();
        let origin = Token::share(&token);
        let time_lock = now + lock_peroid;
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        Token::deposit(&mut token_pool.token, token);
        FixedTimeLockKey<TokenType> {
            origin,
            time_lock,
        }
    }

    // Unlock token by LinearTimeLockKey
    public fun unlock_by_linear<TokenType>(key: &mut LinearTimeLockKey<TokenType>): Token<TokenType> acquires TokenPool {
        let value = unlocked_value_of(key);
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        let token = Token::withdraw_share(&mut token_pool.token, value);
        key.taked = key.taked + value;
        token
    }

    // Unlock token by FixedTimeLockKey
    public fun unlock_by_fixed<TokenType>(key: FixedTimeLockKey<TokenType>): Token<TokenType>  acquires TokenPool {
        let now = Timestamp::now_seconds();
        assert(now >= key.time_lock, ETIMELOCK_NOT_UNLOCKED());
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        let token = Token::withdraw_share(&mut token_pool.token, key.origin);
        let FixedTimeLockKey { origin: _, time_lock: _ } = key;
        token
    }

    // Returns the unlocked value of the LinearTimeLockKey.
    // It represent how much Token in the TokenLockPool can bean withdraw by the key now.
    public fun unlocked_value_of<TokenType>(key: &LinearTimeLockKey<TokenType>): u128 {
        let now = Timestamp::now_seconds();
        let elapsed_time = now - key.lock_time;
        if (elapsed_time >= key.lock_peroid) {
            return key.origin - key.taked
        }else {
            //for avoid overflow
            if (key.origin > (key.lock_peroid as u128)) {
                key.origin / (key.lock_peroid as u128) * (elapsed_time as u128) - key.taked
            }else {
                key.origin * (elapsed_time as u128) / (key.lock_peroid as u128) - key.taked
            }
        }
    }

    public fun time_lock_of<TokenType>(key: &FixedTimeLockKey<TokenType>): u64 {
        key.time_lock
    }

    public fun destroy_empty<TokenType>(key: LinearTimeLockKey<TokenType>) {
        let LinearTimeLockKey<TokenType> { origin, taked, lock_time: _, lock_peroid: _ } = key;
        assert(origin == taked, EDESTROY_KEY_NOT_EMPTY());
    }

    // Save LinearTimeLockKey to sender account.
    public fun save_linear_key<TokenType>(account: &signer, key: LinearTimeLockKey<TokenType>) {
        move_to(account, key);
    }

    // Take LinearTimeLockKey from sender account.
    public fun take_linear_key<TokenType>(account: &signer): LinearTimeLockKey<TokenType> acquires LinearTimeLockKey {
        move_from<LinearTimeLockKey<TokenType>>(Signer::address_of(account))
    }

    public fun exists_linear_key_at<TokenType>(address: address): bool {
        exists<LinearTimeLockKey<TokenType>>(address)
    }

    // Save FixedTimeLockKey to sender account.
    public fun save_fixed_key<TokenType>(account: &signer, key: FixedTimeLockKey<TokenType>) {
        move_to(account, key);
    }

    // Take FixedTimeLockKey from sender account.
    public fun take_fixed_key<TokenType>(account: &signer): FixedTimeLockKey<TokenType> acquires FixedTimeLockKey {
        move_from<FixedTimeLockKey<TokenType>>(Signer::address_of(account))
    }

    public fun exists_fixed_key_at<TokenType>(address: address): bool {
        exists<FixedTimeLockKey<TokenType>>(address)
    }
}
}