// Test the token mint key
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
        let key = Token::issue_fixed_mint_key<STC>(&cap, 0, 5); //amount should large than 0
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
        let key = Token::issue_fixed_mint_key<STC>(&cap, 10000, 0); //period should large than 0
        Token::add_mint_capability(&account, cap);
        Offer::create(&account, key, {{bob}}, 0);
    }
}
// check: "Keep(ABORTED { code: 4615"

// Minting from a privileged account should work
//! new-transaction
//! sender: genesis
script {
    use 0x1::Token;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_key(account: signer) {
        let cap = Token::remove_mint_capability<STC>(&account);
        let key = Token::issue_fixed_mint_key<STC>(&cap, 10000, 5);
        Token::add_mint_capability(&account, cap);
        Offer::create(&account, key, {{bob}}, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Token::{FixedTimeMintKey};
    use 0x1::Collection;

    fun redeem_offer(account: signer) {
        let key = Offer::redeem<FixedTimeMintKey<STC>>(&account, {{genesis}});
        Collection::put(&account,key);
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
    use 0x1::Token::{Self, FixedTimeMintKey};
    use 0x1::Collection;

    fun mint(account: signer) {
        let key = Collection::take<FixedTimeMintKey<STC>>(&account);
        let token = Token::mint_with_fixed_key(key); //EMINT_AMOUNT_EQUAL_ZERO
        Account::deposit_to_self(&account, token);
    }
}

// check: "Keep(ABORTED { code: 27911"


//! block-prologue
//! author: alice
//! block-time: 5000
//! block-number: 2

//! new-transaction
//! sender: bob
script {
    use 0x1::STC::STC;
    use 0x1::Token::{Self, FixedTimeMintKey};
    use 0x1::Collection;

    fun mint(account: signer) {
        let key = Collection::take<FixedTimeMintKey<STC>>(&account);
        assert(Token::end_time_of_key<STC>(&key) == 5, 1001); //5 seconds
        Collection::put(&account,key);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token::{Self, FixedTimeMintKey};
    use 0x1::Collection;

    fun mint(account: signer) {
        let key = Collection::take<FixedTimeMintKey<STC>>(&account);
        let token = Token::mint_with_fixed_key(key);
        assert(Token::value(&token) == 10000, 1001);
        Account::deposit_to_self(&account, token);
    }
}