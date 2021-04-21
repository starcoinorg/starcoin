//! account: alice, 0 0x1::STC::STC

//! sender: association
script {
    use 0x1::STC::STC;
    use 0x1::Treasury;
    //use 0x1::Debug;

    fun mint(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        assert(Treasury::get_linear_withdraw_capability_withdraw(&cap) == 0, 1001);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//! block-prologue
//! author: alice
//! block-time: 10000
//! block-number: 1

//! new-transaction
//! sender: association
script {
    use 0x1::TreasuryScripts;
    use 0x1::STC::STC;

    fun main(account: signer) {
        TreasuryScripts::withdraw_and_split_lt_withdraw_cap<STC>(account, {{alice}}, 100000000000000, 0);
    }
}

// check: gas_used
// check: 213425
// check: "Keep(EXECUTED)"

//! block-prologue
//! author: alice
//! block-time: 20000
//! block-number: 2


//! new-transaction
//! sender: alice
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Treasury;

    fun redeem_offer(account: signer) {
        let cap = Offer::redeem<Treasury::LinearTimeWithdrawCapability<STC>>(&account, {{association}});
        Treasury::add_linear_withdraw_capability(&account,cap);
    }
}


//! block-prologue
//! author: alice
//! block-time: 60000
//! block-number: 3

//! new-transaction
//! sender: alice
script {
    use 0x1::TreasuryScripts;
    use 0x1::STC::STC;

    fun main(account: signer) {
        TreasuryScripts::withdraw_token_with_linear_withdraw_capability<STC>(account);
    }
}
// check: gas_used
// check: 167346
// check: "Keep(EXECUTED)"
