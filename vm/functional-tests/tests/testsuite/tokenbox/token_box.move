// Test the token offer
//! account: alice, 100000 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

//! sender: alice
script {
    use 0x1::Account;
    use 0x1::TokenBox;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_box(account: &signer) {
        let token = Account::withdraw<STC>(account, 10000);
        let box = TokenBox::create<STC>(token, 5);
        Offer::create(account, box, {{bob}}, 0);
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::TokenBox::{Self, TokenBox};

    fun redeem_offer(account: &signer) {
        let box = Offer::redeem<TokenBox<STC>>(account, {{alice}});
        TokenBox::save(account, box);
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
    use 0x1::TokenBox;

    fun unlock_tokenbox(account: &signer) {
        let box = TokenBox::take<STC>(account);
        let token = TokenBox::withdraw(&mut box);
        // withdraw 10000/5
        assert(Token::share(&token) == 2000, 101);
        TokenBox::save(account, box);
        Account::deposit(account, token);
    }
}

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
    use 0x1::TokenBox;

    fun unlock_tokenbox(account: &signer) {
        let box = TokenBox::take<STC>(account);
        let token = TokenBox::withdraw(&mut box);
        assert(Token::share(&token) == 8000, 102);
        TokenBox::destroy_empty(box);
        Account::deposit(account, token);
    }
}