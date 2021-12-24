//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr carol

//# run --signers alice
script {
    use Std::Account;
    use Std::Offer;
    use Std::STC::STC;
    use Std::Signer;
    use Std::Token::Token;

    fun create_offer(account: signer) {
        let token = Account::withdraw<STC>(&account, 10000);
        Offer::create(&account, token, @bob, 5);
        // test Offer::exists_at
        assert!(Offer::exists_at<Token<STC>>(Signer::address_of(&account)), 1001);
        // test Offer::address_of
        assert!(Offer::address_of<Token<STC>>(Signer::address_of(&account)) == @bob, 1002);
    }
}

// check: EXECUTED

//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1



//# run --signers bob
script {
    use Std::Account;
    use Std::Offer;
    use Std::Token::Token;
    use Std::STC::STC;

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, @alice);
        Account::deposit_to_self(&account, token);
    }
}

// check: "Keep(ABORTED { code: 26117"

//# block --author alice

//# run --signers carol
script {
    use Std::Account;
    use Std::Offer;
    use Std::Token::Token;
    use Std::STC::STC;

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, @alice);
        Account::deposit_to_self(&account, token);
    }
}
// check: "Keep(ABORTED { code: 25863"

//# block --author alice

//# run --signers bob
script {
    use Std::Account;
    use Std::Offer;
    use Std::Token::Token;
    use Std::STC::STC;

    fun redeem_offer(account: signer) {
        let token = Offer::redeem<Token<STC>>(&account, @alice);
        Account::deposit_to_self(&account, token);
    }
}

// check: EXECUTED
