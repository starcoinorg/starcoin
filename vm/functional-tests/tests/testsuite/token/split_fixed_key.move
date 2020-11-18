// Test split fixed mint key
//! account: alice, 0 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

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
    use 0x1::Box;
    use 0x1::Token::{FixedTimeMintKey};

    fun bob_take_fixed_key_from_offer(account: &signer) {
        let key = Offer::redeem<FixedTimeMintKey<STC>>(account, {{genesis}});
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
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Box;
    use 0x1::Token;

    fun split_fixed_key(signer: &signer) {
        let mint_key = Box::take<Token::FixedTimeMintKey<STC>>(signer);
        let new_mint_key = Token::split_fixed_key<STC>(&mut mint_key, 200);
        Box::put(signer, mint_key);
        Offer::create<Token::FixedTimeMintKey<STC>>(signer, new_mint_key, {{alice}}, 0);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Box;
    use 0x1::Token::{FixedTimeMintKey};

    fun alice_take_fixed_key_from_offer(account: &signer) {
        let key = Offer::redeem<FixedTimeMintKey<STC>>(account, {{bob}});
        Box::put(account, key);
    }
}

//! block-prologue
//! author: bob
//! block-time: 10000
//! block-number: 2

//! new-transaction
//! sender: alice
script {
    use 0x1::STC::STC;
    use 0x1::Box;
    use 0x1::Token;
    use 0x1::Account;

    fun alice_mint_by_fixed_key(signer: &signer) {
        let mint_key = Box::take<Token::FixedTimeMintKey<STC>>(signer);
        let tokens = Token::mint_with_fixed_key<STC>(mint_key);
        assert(Token::value(&tokens) > 0, 102);
        Account::deposit_to_self(signer, tokens);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::STC::STC;
    use 0x1::Box;
    use 0x1::Token;
    use 0x1::Account;

    fun bob_mint_by_fixed_key(signer: &signer) {
        let mint_key = Box::take<Token::FixedTimeMintKey<STC>>(signer);
        let tokens = Token::mint_with_fixed_key<STC>(mint_key);
        assert(Token::value(&tokens) > 0, 103);
        Account::deposit_to_self(signer, tokens);
    }
}