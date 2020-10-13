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
    use 0x1::Token;
    use 0x1::Box;
    use 0x1::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        // withdraw 10000/5
        assert(Token::share(&token) == 2000, 1001);
        Box::put(account, key);
        Account::deposit(account, token);
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
    use 0x1::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        // withdraw 10000/5 again
        assert(Token::share(&token) == 2000, 1002);
        Box::put(account, key);
        Account::deposit(account, token);
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
    use 0x1::TokenLockPool::{Self, LinearTimeLockKey};

    fun unlock(account: &signer) {
        let key = Box::take<LinearTimeLockKey<STC>>(account);
        //unlock all remain
        let token = TokenLockPool::unlock_with_linear_key(&mut key);
        assert(Token::share(&token) == 6000, 1003);
        TokenLockPool::destroy_empty(key);
        Account::deposit(account, token);
    }
}