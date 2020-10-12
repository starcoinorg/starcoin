// Test the token lock
//! account: alice, 100000 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

//! sender: alice
script {
    use 0x1::Account;
    use 0x1::TokenLockPool;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_lock(account: &signer) {
        let token = Account::withdraw<STC>(account, 10000);
        let key = TokenLockPool::create_fixed_lock<STC>(token, 5);
        Offer::create(account, key, {{bob}}, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::TokenLockPool::{FixedTimeLockKey};
    use 0x1::Box;

    fun redeem_offer(account: &signer) {
        let key = Offer::redeem<FixedTimeLockKey<STC>>(account, {{alice}});
        Box::put(account,key);
    }
}

//! block-prologue
//! author: alice
//! block-time: 1
//! block-number: 1

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::TokenLockPool::{Self, FixedTimeLockKey};
    use 0x1::Box;

    fun unlock(account: &signer) {
        let key = Box::take<FixedTimeLockKey<STC>>(account);
        let token = TokenLockPool::unlock_with_fixed_key(key);
        Account::deposit(account, token);
    }
}

// check: ABORTED


//! block-prologue
//! author: alice
//! block-time: 5
//! block-number: 2

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token;
    use 0x1::TokenLockPool::{Self, FixedTimeLockKey};
    use 0x1::Box;

    fun unlock(account: &signer) {
        let key = Box::take<FixedTimeLockKey<STC>>(account);
        let token = TokenLockPool::unlock_with_fixed_key(key);
        assert(Token::value(&token) == 10000, 1001);
        Account::deposit(account, token);
    }
}