// Test the token lock
//! account: alice, 100000 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

module TokenLockPool {
    use 0x1::Token::{Self, Token};
    use 0x1::Timestamp;
    use 0x1::CoreAddresses;
    use 0x1::STC::STC;
    use 0x1::Errors;
    use 0x1::Math;
    use 0x1::Signer;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    // A global pool for lock token.
    resource struct TokenPool<TokenType> { token: Token<TokenType> }

    // A fixed time lock key which can withdraw locked token until global time > end time
    resource struct FixedTimeLockKey<TokenType> { total: u128, end_time: u64 }

    // A linear time lock key which can withdraw locked token in a peroid by time-based linear release.
    resource struct LinearTimeLockKey<TokenType> { total: u128, taked: u128, start_time: u64, peroid: u64 }

    const EINVALID_ARGUMENT: u64 = 18;
    // The key which to destory is not empty.
    const EDESTROY_KEY_NOT_EMPTY: u64 = 101;

    // Timelock is not unlocked yet.
    const ETIMELOCK_NOT_UNLOCKED: u64 = 102;

    // Amount too big than locked token's value.
    const EAMOUNT_TOO_BIG: u64 = 103;

    public fun initialize(account: &signer) {
        CoreAddresses::assert_genesis_address(account);
        let token_pool = TokenPool<STC> { token: Token::zero() };
        move_to(account, token_pool);
        //TODO how to init other token's pool.
    }

    spec fun initialize {
        include CoreAddresses::AbortsIfNotGenesisAddress;
        aborts_if exists<TokenPool<STC>>(Signer::address_of(account));
    }

    // Create a LinearTimeLock by token and peroid in seconds.
    public fun create_linear_lock<TokenType>(token: Token<TokenType>, peroid: u64): LinearTimeLockKey<TokenType> acquires TokenPool {
        assert(peroid > 0, Errors::invalid_argument(EINVALID_ARGUMENT));
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

    spec fun create_linear_lock {
        aborts_if peroid <= 0;
        include Timestamp::AbortsIfTimestampNotExists;
        aborts_if !exists<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        aborts_if global<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS()).token.value + token.value > max_u128();
    }

    // Create a FixedTimeLock by token and peroid in seconds.
    public fun create_fixed_lock<TokenType>(token: Token<TokenType>, peroid: u64): FixedTimeLockKey<TokenType> acquires TokenPool {
        assert(peroid > 0, Errors::invalid_argument(EINVALID_ARGUMENT));
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

    spec fun create_fixed_lock {
        aborts_if peroid <= 0;
        include Timestamp::AbortsIfTimestampNotExists;
        aborts_if Timestamp::now_seconds() + peroid > max_u64();
        aborts_if !exists<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        aborts_if global<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS()).token.value + token.value > max_u128();
    }

    // Unlock token with LinearTimeLockKey
    public fun unlock_with_linear_key<TokenType>(key: &mut LinearTimeLockKey<TokenType>): Token<TokenType> acquires TokenPool {
        let amount = unlocked_amount_of_linear_key(key);
        assert(amount > 0, Errors::invalid_state(ETIMELOCK_NOT_UNLOCKED));
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        let token = Token::withdraw(&mut token_pool.token, amount);
        key.taked = key.taked + amount;
        token
    }

    spec fun unlock_with_linear_key {
        //aborts_if unlocked_amount_of_linear_key(key) <= 0;
        aborts_if !exists<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        aborts_if global<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS()).token.value < key.total;
        include Timestamp::AbortsIfTimestampNotExists;
        //aborts_if unlocked_amount_of_linear_key(old(key)) + key.taked  > max_u128();
        pragma verify = false;// fix me
    }

    // Unlock token with FixedTimeLockKey
    public fun unlock_with_fixed_key<TokenType>(key: FixedTimeLockKey<TokenType>): Token<TokenType>  acquires TokenPool {
        let amount = unlocked_amount_of_fixed_key(&key);
        assert(amount > 0, Errors::invalid_state(ETIMELOCK_NOT_UNLOCKED));
        let token_pool = borrow_global_mut<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        let token = Token::withdraw(&mut token_pool.token, key.total);
        let FixedTimeLockKey { total: _, end_time: _ } = key;
        token
    }

    spec fun unlock_with_fixed_key {
        aborts_if unlocked_amount_of_fixed_key(key) <= 0;
        aborts_if !exists<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS());
        aborts_if global<TokenPool<TokenType>>(CoreAddresses::GENESIS_ADDRESS()).token.value < key.total;
        include Timestamp::AbortsIfTimestampNotExists;
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

    spec fun unlocked_amount_of_linear_key {
        include Timestamp::AbortsIfTimestampNotExists;
        //aborts_if key.total - key.taked > max_u128();
        pragma verify = false;// fix me
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

    spec fun unlocked_amount_of_fixed_key {
        include Timestamp::AbortsIfTimestampNotExists;
    }

    public fun end_time_of<TokenType>(key: &FixedTimeLockKey<TokenType>): u64 {
        key.end_time
    }

    spec fun end_time_of {aborts_if false;}

    public fun destroy_empty<TokenType>(key: LinearTimeLockKey<TokenType>) {
        let LinearTimeLockKey<TokenType> { total, taked, start_time: _, peroid: _ } = key;
        assert(total == taked, Errors::invalid_state(EDESTROY_KEY_NOT_EMPTY));
    }

    spec fun destroy_empty {
        aborts_if key.total != key.taked;
    }

}

//! new-transaction
//!sender: genesis
script {
    use {{default}}::TokenLockPool;
    fun init(account: &signer){
        TokenLockPool::initialize(account);
    }
}

//! new-transaction

//! sender: alice
script {
    use 0x1::Account;
    use {{default}}::TokenLockPool;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_lock(account: &signer) {
        let token = Account::withdraw<STC>(account, 10000);
        let key = TokenLockPool::create_linear_lock<STC>(token, 5);
        Offer::create(account, key, {{bob}}, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Box;
    use {{default}}::TokenLockPool::{LinearTimeLockKey};

    fun redeem_offer(account: &signer) {
        let key = Offer::redeem<LinearTimeLockKey<STC>>(account, {{alice}});
        Box::put(account, key);
    }
}


//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token;
    use 0x1::Box;
    use {{default}}::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        // withdraw 10000/5
        assert(Token::value(&token) == 2000, 1001);
        Box::put(account, key);
        Account::deposit_to_self(account, token);
    }
}

//! block-prologue
//! author: alice
//! block-time: 2000
//! block-number: 2

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token;
    use 0x1::Box;
    use {{default}}::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        // withdraw 10000/5 again
        assert(Token::value(&token) == 2000, 1002);
        Box::put(account, key);
        Account::deposit_to_self(account, token);
    }
}

//! block-prologue
//! author: alice
//! block-time: 5000
//! block-number: 3

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token;
    use 0x1::Box;
    use {{default}}::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        //unlock all remain
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        assert(Token::value(&token) == 6000, 1003);
        TokenLockPool::destroy_empty(key);
        Account::deposit_to_self(account, token);
    }
}