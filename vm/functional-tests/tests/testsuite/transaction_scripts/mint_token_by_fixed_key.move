//! account: alice, 0 0x1::STC::STC
//! account: bob, 0 0x1::STC::STC

//! new-transaction
//! sender: genesis
script {
    use 0x1::Token;
    use 0x1::STC::STC;
    use 0x1::Offer;

    fun create_key(account: signer) {
        let cap = Token::remove_mint_capability<STC>(&account);
        let key = Token::issue_fixed_mint_key<STC>(&cap, 10000, 2);
        Token::add_mint_capability(&account, cap);
        Offer::create(&account, key, {{bob}}, 0);
    }
}
// check: gas_used
// check: 63115

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
// check: gas_used
// check: 85093

//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1

//! block-prologue
//! author: alice
//! block-time: 2000
//! block-number: 2

//! new-transaction
//! sender: bob
script {
    use 0x1::MintScripts;
    use 0x1::STC::STC;

    fun main(account: signer) {
        MintScripts::mint_token_by_fixed_key<STC>(account);
    }
}
// check: gas_used
// check: 198126
// check: "Keep(EXECUTED)"

