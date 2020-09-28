// Test the token offer
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
    use 0x1::TokenLockPool::{Self, LinearTimeLockKey};

    fun redeem_offer(account: &signer) {
        let key = Offer::redeem<LinearTimeLockKey<STC>>(account, {{alice}});
        TokenLockPool::save_linear_key(account, key);
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
    use 0x1::Token;
    use 0x1::TokenLockPool;

    fun unlock(account: &signer) {
        let key = TokenLockPool::take_linear_key<STC>(account);
        let token = TokenLockPool::unlock_by_linear(&mut key);
        // withdraw 10000/5
        assert(Token::share(&token) == 2000, 1001);
        TokenLockPool::save_linear_key(account, key);
        Account::deposit(account, token);
    }
}

//! block-prologue
//! author: alice
//! block-time: 2
//! block-number: 2

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token;
    use 0x1::TokenLockPool;

    fun unlock(account: &signer) {
        let key = TokenLockPool::take_linear_key<STC>(account);
        let token = TokenLockPool::unlock_by_linear(&mut key);
        // withdraw 10000/5 again
        assert(Token::share(&token) == 2000, 1002);
        TokenLockPool::save_linear_key(account, key);
        Account::deposit(account, token);
    }
}

//! block-prologue
//! author: alice
//! block-time: 5
//! block-number: 3

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token;
    use 0x1::TokenLockPool;

    fun unlock(account: &signer) {
        let key = TokenLockPool::take_linear_key<STC>(account);
        //unlock all remain
        let token = TokenLockPool::unlock_by_linear(&mut key);
        assert(Token::share(&token) == 6000, 1003);
        TokenLockPool::destroy_empty(key);
        Account::deposit(account, token);
    }
}