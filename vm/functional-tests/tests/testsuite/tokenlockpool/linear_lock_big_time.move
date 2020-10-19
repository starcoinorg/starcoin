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
        let token = Account::withdraw<STC>(account, 100);
        let key = TokenLockPool::create_linear_lock<STC>(token, 10000);
        Offer::create(account, key, {{bob}}, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Box;
    use 0x1::TokenLockPool::{LinearTimeLockKey};

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
    use 0x1::Box;
    use 0x1::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        Box::put(account, key);
        Account::deposit_to_self(account, token);
    }
}

// check: ABORTED

//! block-prologue
//! author: alice
//! block-time: 100000
//! block-number: 2

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token;
    use 0x1::Box;
    use 0x1::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        assert(Token::value(&token) == 1, 1002);
        Box::put(account, key);
        Account::deposit_to_self(account, token);
    }
}


//! block-prologue
//! author: alice
//! block-time: 10000000
//! block-number: 3

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token;
    use 0x1::Box;
    use 0x1::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        //unlock all remain
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        assert(Token::value(&token) == 99, 1003);
        TokenLockPool::destroy_empty(key);
        Account::deposit_to_self(account, token);
    }
}