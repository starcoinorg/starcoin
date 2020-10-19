// Test the token mint key
//! account: alice, 0 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

// Minting from a privileged account should work
//! sender: genesis
script {
    use 0x1::Token;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_key(account: &signer) {
        let cap = Token::remove_mint_capability<STC>(account);
        let key = Token::issue_fixed_mint_key<STC>(&cap, 10000, 5);
        Token::add_mint_capability(account, cap);
        Offer::create(account, key, {{bob}}, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Token::{FixedTimeMintKey};
    use 0x1::Box;

    fun redeem_offer(account: &signer) {
        let key = Offer::redeem<FixedTimeMintKey<STC>>(account, {{genesis}});
        Box::put(account,key);
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
    use 0x1::Box;

    fun mint(account: &signer) {
        let key = Box::take<FixedTimeMintKey<STC>>(account);
        let token = Token::mint_with_fixed_key(key);
        Account::deposit_to_self(account, token);
    }
}

// check: ABORTED


//! block-prologue
//! author: alice
//! block-time: 5000
//! block-number: 2

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::STC::STC;
    use 0x1::Token::{Self, FixedTimeMintKey};
    use 0x1::Box;

    fun mint(account: &signer) {
        let key = Box::take<FixedTimeMintKey<STC>>(account);
        let token = Token::mint_with_fixed_key(key);
        assert(Token::value(&token) == 10000, 1001);
        Account::deposit_to_self(account, token);
    }
}