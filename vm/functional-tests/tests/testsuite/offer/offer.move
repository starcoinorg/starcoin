// Test offer
//! account: alice, 100000 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

//! sender: alice
script {
    use 0x1::Account;
    use 0x1::Offer;
    use 0x1::STC::STC;

    fun create_offer(account: &signer) {
        let token = Account::withdraw<STC>(account, 10000);
        Offer::create(account, token, {{bob}}, 5);
    }
}

// check: EXECUTED

//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::Offer;
    use 0x1::Token::Token;
    use 0x1::STC::STC;

    fun redeem_offer(account: &signer) {
        let token = Offer::redeem<Token<STC>>(account, {{alice}});
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
    use 0x1::Offer;
    use 0x1::Token::Token;
    use 0x1::STC::STC;

    fun redeem_offer(account: &signer) {
        let token = Offer::redeem<Token<STC>>(account, {{alice}});
        Account::deposit_to_self(account, token);
    }
}

// check: EXECUTED
