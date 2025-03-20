//# init -n dev

//# faucet --addr alice --amount 0

//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Treasury;
    //use StarcoinFramework::Debug;

    fun mint(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        assert!(Treasury::get_linear_withdraw_capability_withdraw(&cap) == 0, 1001);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//# block --author alice


//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::TreasuryScripts;
    use StarcoinFramework::STC::STC;

    fun main(account: signer) {
        TreasuryScripts::withdraw_and_split_lt_withdraw_cap<STC>(account, @alice, 100000000000000, 0);
    }
}


//# block --author alice


//# run --signers alice
script {
    use StarcoinFramework::Offer;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Treasury;

    fun redeem_offer(account: signer) {
        let cap = Offer::redeem<Treasury::LinearWithdrawCapability<STC>>(&account, @StarcoinAssociation);
        Treasury::add_linear_withdraw_capability(&account,cap);
    }
}


//# block --author alice


//# run --signers alice

script {
    use StarcoinFramework::TreasuryScripts;
    use StarcoinFramework::STC::STC;

    fun main(account: signer) {
        TreasuryScripts::withdraw_token_with_linear_withdraw_capability<STC>(account);
    }
}
