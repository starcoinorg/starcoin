// Test the token mint
//! account: alice, 0 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

//! sender: genesis
script {
    use 0x1::Token;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_key(account: &signer) {
        let cap = Token::remove_mint_capability<STC>(account);
        let key = Token::issue_linear_mint_key<STC>(&cap, 10000, 5);
        Token::add_mint_capability(account, cap);
        Offer::create(account, key, {{bob}}, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Box;
    use 0x1::Token::{LinearTimeMintKey};

    fun redeem_offer(account: &signer) {
        let key = Offer::redeem<LinearTimeMintKey<STC>>(account, {{genesis}});
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
    use 0x1::Token::{Self, LinearTimeMintKey};

    fun mint(account: &signer) {
        let key = Box::take<LinearTimeMintKey<STC>>(account);
        let token = Token::mint_with_linear_key(&mut key);
        // mint 10000/5
        assert(Token::value(&token) == 2000, 1001);
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
    use 0x1::Box;
    use 0x1::Token::{Self, LinearTimeMintKey};
    use 0x1::Debug;

    fun mint(account: &signer) {
        let key = Box::take<LinearTimeMintKey<STC>>(account);
        let token = Token::mint_with_linear_key(&mut key);
        Debug::print(&Token::value(&token));
        // mint 10000/5 again
        assert(Token::value(&token) == 2000, 1002);
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
    use 0x1::Box;
    use 0x1::Token::{Self, LinearTimeMintKey};

    fun mint(account: &signer) {
        let key = Box::take<LinearTimeMintKey<STC>>(account);
        //mint all remain
        let token = Token::mint_with_linear_key(&mut key);
        assert(Token::value(&token) == 6000, 1003);
        Token::destroy_empty_key(key);
        Account::deposit(account, token);
    }
}