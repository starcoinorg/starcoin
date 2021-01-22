//! account: alice, 0 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

//! new-transaction
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
// check: gas_used
// check: 64379

//! new-transaction
//! sender: bob
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Box;
    use 0x1::Token::{LinearTimeMintKey};

    fun redeem_offer(account: &signer) {
        let key = Offer::redeem<LinearTimeMintKey<STC>>(account, {{genesis}});
        Box::put(account, key);
    }
}
// check: gas_used
// check: 77479

//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1

//! new-transaction
//! sender: bob
//! type-args: 0x1::STC::STC
stdlib_script::mint_token_by_linear_key
// check: gas_used
// check: 173305
// check: "Keep(EXECUTED)"
