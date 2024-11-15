//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr carol

//# run --signers alice
script {
    use starcoin_framework::account;
    use starcoin_framework::Offer;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::signer;
    use starcoin_framework::Token::Token;

    fun create_offer(account: signer) {
        let token = coin::withdraw<STC>(&account, 10000);
        Offer::create(&account, token, @bob, 5);
        // test Offer::exists_at
        assert!(Offer::exists_at<Token<STC>>(signer::address_of(&account)), 1001);
        // test Offer::address_of
        assert!(Offer::address_of<Token<STC>>(signer::address_of(&account)) == @bob, 1002);
    }
}

// check: EXECUTED

//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1



//# run --signers bob
script {
    use starcoin_framework::account;
    use starcoin_framework::Offer;
    use starcoin_framework::Token::Token;
    use starcoin_framework::starcoin_coin::STC;

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, @alice);
        coin::deposit(&account, token);
    }
}

// check: "Keep(ABORTED { code: 26117"

//# block --author alice

//# run --signers carol
script {
    use starcoin_framework::account;
    use starcoin_framework::Offer;
    use starcoin_framework::Token::Token;
    use starcoin_framework::starcoin_coin::STC;

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, @alice);
        coin::deposit(&account, token);
    }
}
// check: "Keep(ABORTED { code: 25863"

//# block --author alice

//# run --signers bob
script {
    use starcoin_framework::account;
    use starcoin_framework::Offer;
    use starcoin_framework::Token::Token;
    use starcoin_framework::starcoin_coin::STC;

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, @alice);
        coin::deposit(&account, token);
    }
}

// check: EXECUTED
