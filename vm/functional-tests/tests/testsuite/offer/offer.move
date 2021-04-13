// Test offer
//! account: alice, 100000 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC
//! account: carol, 0 0x1::STC::STC

//! sender: alice
script {
    use 0x1::Account;
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Signer;
    use 0x1::Token::Token;

    fun create_offer(account: signer) {
        let token = Account::withdraw<STC>(&account, 10000);
        Offer::create(&account, token, {{bob}}, 5);
        // test Offer::exists_at
        assert(Offer::exists_at<Token<STC>>(Signer::address_of(&account)), 1001);
        // test Offer::address_of
        assert(Offer::address_of<Token<STC>>(Signer::address_of(&account)) == {{bob}}, 1002);
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

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, {{alice}});
        Account::deposit_to_self(&account, token);
    }
}

// check: "Keep(ABORTED { code: 26117"

//! block-prologue
//! author: alice
//! block-time: 5000
//! block-number: 2

//! new-transaction
//! sender: carol
script {
    use 0x1::Account;
    use 0x1::Offer;
    use 0x1::Token::Token;
    use 0x1::STC::STC;

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, {{alice}});
        Account::deposit_to_self(&account, token);
    }
}
// check: "Keep(ABORTED { code: 25863"

//! new-transaction
//! sender: bob
script {
    use 0x1::Account;
    use 0x1::Offer;
    use 0x1::Token::Token;
    use 0x1::STC::STC;

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, {{alice}});
        Account::deposit_to_self(&account, token);
    }
}

// check: EXECUTED
