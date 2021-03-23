// Test split linear mint key
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
    use 0x1::Collection;
    use 0x1::Token::{LinearTimeMintKey};

    fun bob_take_linear_key_from_offer(account: &signer) {
        let key = Offer::redeem<LinearTimeMintKey<STC>>(account, {{genesis}});
        Collection::put(account, key);
    }
}

//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1

// split linear key with wrong parameter
//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token;
    use 0x1::Account;

    fun split_linear_key(signer: &signer) {
        let mint_key = Collection::take<Token::LinearTimeMintKey<STC>>(signer);
        let (tokens, new_mint_key) = Token::split_linear_key<STC>(&mut mint_key, 20000);
        Account::deposit_to_self(signer, tokens);
        Collection::put(signer, mint_key);
        Offer::create<Token::LinearTimeMintKey<STC>>(signer, new_mint_key, {{alice}}, 0);
    }
}
// check: "Keep(ABORTED { code: 27393"

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token;
    use 0x1::Account;

    fun split_linear_key(signer: &signer) {
        let mint_key = Collection::take<Token::LinearTimeMintKey<STC>>(signer);
        let (tokens, new_mint_key) = Token::split_linear_key<STC>(&mut mint_key, 200);
        assert(Token::value(&tokens) > 0, 100);
        assert(Token::value(&tokens) < 10000, 101);
        Account::deposit_to_self(signer, tokens);
        Collection::put(signer, mint_key);
        Offer::create<Token::LinearTimeMintKey<STC>>(signer, new_mint_key, {{alice}}, 0);
    }
}

//! new-transaction
//! sender: alice
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token::{LinearTimeMintKey};

    fun alice_take_linear_key_from_offer(account: &signer) {
        let key = Offer::redeem<LinearTimeMintKey<STC>>(account, {{bob}});
        Collection::put(account, key);
    }
}

//! block-prologue
//! author: bob
//! block-time: 2000
//! block-number: 2

//! new-transaction
//! sender: alice
script {
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token;
    use 0x1::Account;

    fun alice_mint_by_linear_key(signer: &signer) {
        let mint_key = Collection::take<Token::LinearTimeMintKey<STC>>(signer);
        let tokens = Token::mint_with_linear_key<STC>(&mut mint_key);
        assert(Token::value(&tokens) > 0, 102);
        Account::deposit_to_self(signer, tokens);
        Collection::put(signer, mint_key);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token;
    use 0x1::Account;

    fun bob_mint_by_linear_key(signer: &signer) {
        let mint_key = Collection::take<Token::LinearTimeMintKey<STC>>(signer);
        let tokens = Token::mint_with_linear_key<STC>(&mut mint_key);
        assert(Token::value(&tokens) > 0, 103);
        Account::deposit_to_self(signer, tokens);
        Collection::put(signer, mint_key);
    }
}

//! block-prologue
//! author: bob
//! block-time: 3000
//! block-number: 3

// split linear key
//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token;
    use 0x1::Account;

    fun split_linear_key_failed(signer: &signer) {
        let mint_key = Collection::take<Token::LinearTimeMintKey<STC>>(signer);
        assert(!Token::is_empty_key(&mint_key), 99);
        let (tokens, new_mint_key) = Token::split_linear_key<STC>(&mut mint_key, 10000);//ESPLIT
        assert(Token::value(&tokens) > 0, 100);
        assert(Token::value(&tokens) < 10000, 101);
        Account::deposit_to_self(signer, tokens);
        Collection::put(signer, mint_key);
        Offer::create<Token::LinearTimeMintKey<STC>>(signer, new_mint_key, {{alice}}, 0);
    }
}
// check: "Keep(ABORTED { code: 27393"

// split linear key
//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token;
    use 0x1::Account;

    fun split_linear_key_failed(signer: &signer) {
        let mint_key = Collection::take<Token::LinearTimeMintKey<STC>>(signer);
        assert(!Token::is_empty_key(&mint_key), 99);
        let (tokens, new_mint_key) = Token::split_linear_key<STC>(&mut mint_key, 100);//ESPLIT
        assert(Token::value(&tokens) > 0, 100);
        assert(Token::value(&tokens) < 10000, 101);
        Account::deposit_to_self(signer, tokens);
        Collection::put(signer, mint_key);
        Offer::create<Token::LinearTimeMintKey<STC>>(signer, new_mint_key, {{alice}}, 0);
    }
}


//! block-prologue
//! author: bob
//! block-time: 6000
//! block-number: 4

// split linear key
//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Collection;
    use 0x1::Token;
    use 0x1::Account;

    fun split_linear_key_failed(signer: &signer) {
        let mint_key = Collection::take<Token::LinearTimeMintKey<STC>>(signer);
        assert(!Token::is_empty_key(&mint_key), 99);
        // key is not empty, but it's time to mint all token, split will be failed
        let (tokens, new_mint_key) = Token::split_linear_key<STC>(&mut mint_key, 100); //EEMPTY_KEY
        assert(Token::value(&tokens) > 0, 100);
        assert(Token::value(&tokens) < 10000, 101);
        Account::deposit_to_self(signer, tokens);
        Collection::put(signer, mint_key);
        Offer::create<Token::LinearTimeMintKey<STC>>(signer, new_mint_key, {{alice}}, 0);
    }
}

// check: "Keep(ABORTED { code: 27137"