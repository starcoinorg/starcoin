// Test the token mint
//! account: alice, 0 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

// issue mint key with wrong parameter
//! sender: genesis
script {
    use 0x1::Token;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_key(account: signer) {
        let cap = Token::remove_mint_capability<STC>(&account);
        let key = Token::issue_linear_mint_key<STC>(&cap, 0, 5); //amount should large than 0
        Token::add_mint_capability(&account, cap);
        Offer::create(&account, key, {{bob}}, 0);
    }
}
// check: "Keep(ABORTED { code: 4615"

// issue mint key with wrong parameter
//! new-transaction
//! sender: genesis
script {
    use 0x1::Token;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_key(account: signer) {
        let cap = Token::remove_mint_capability<STC>(&account);
        let key = Token::issue_linear_mint_key<STC>(&cap, 10000, 0); //period should large than 0
        Token::add_mint_capability(&account, cap);
        Offer::create(&account, key, {{bob}}, 0);
    }
}
// check: "Keep(ABORTED { code: 4615"

//! new-transaction
//! sender: genesis
script {
    use 0x1::Token;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_key(account: signer) {
        let cap = Token::remove_mint_capability<STC>(&account);
        let key = Token::issue_linear_mint_key<STC>(&cap, 10000, 5);
        Token::add_mint_capability(&account, cap);
        Offer::create(&account, key, {{bob}}, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token::{LinearTimeMintKey};

    fun redeem_offer(account: signer) {
        let key = Offer::redeem<LinearTimeMintKey<STC>>(&account, {{genesis}});
        Collection::put(&account, key);
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
    use 0x1::Collection;
    use 0x1::Token::{Self, LinearTimeMintKey};

    fun mint(account: signer) {
        let key = Collection::take<LinearTimeMintKey<STC>>(&account);
        let token = Token::mint_with_linear_key(&mut key);
        // mint 10000/5
        assert(Token::value(&token) == 2000, 1001);
        Collection::put(&account, key);
        Account::deposit_to_self(&account, token);
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
    use 0x1::Collection;
    use 0x1::Token::{Self, LinearTimeMintKey};
    use 0x1::Debug;

    fun mint(account: signer) {
        let key = Collection::take<LinearTimeMintKey<STC>>(&account);
        let token = Token::mint_with_linear_key(&mut key);
        Debug::print(&Token::value(&token));
        // mint 10000/5 again
        assert(Token::value(&token) == 2000, 1002);
        Collection::put(&account, key);
        Account::deposit_to_self(&account, token);
    }
}

//! block-prologue
//! author: alice
//! block-time: 5000
//! block-number: 3

//! new-transaction
//! sender: bob
script {
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token::{Self, LinearTimeMintKey};

    fun mint(account: signer) {
        let key = Collection::take<LinearTimeMintKey<STC>>(&account);
        Token::destroy_empty_key(key); //EDESTROY_KEY_NOT_EMPTY
    }
}
// check: "Keep(ABORTED { code: 26631"

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token::{Self, LinearTimeMintKey};

    fun mint(account: signer) {
        let key = Collection::take<LinearTimeMintKey<STC>>(&account);
        //mint all remain
        let token = Token::mint_with_linear_key(&mut key);
        assert(Token::value(&token) == 6000, 1003);
        Account::deposit_to_self(&account, token);
        Collection::put(&account, key);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token::{Self, LinearTimeMintKey};

    fun mint(account: signer) {
        let key = Collection::take<LinearTimeMintKey<STC>>(&account);
        //mint empty
        let token = Token::mint_with_linear_key(&mut key); //EMINT_AMOUNT_EQUAL_ZERO
        Account::deposit_to_self(&account, token);
        Collection::put(&account, key);
    }
}
// check: "Keep(ABORTED { code: 27911"

//! new-transaction
//! sender: bob
script {
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token::{Self, LinearTimeMintKey};

    fun mint(account: signer) {
        let key = Collection::take<LinearTimeMintKey<STC>>(&account);
        Token::destroy_empty_key(key);
    }
}