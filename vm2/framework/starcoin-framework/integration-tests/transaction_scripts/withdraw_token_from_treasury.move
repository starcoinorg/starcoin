//# init -n dev

//# faucet --addr alice --amount 0

//# run --signers StarcoinAssociation
script {
    use starcoin_framework::treasury;
    use starcoin_framework::starcoin_coin::STC;

    fun mint(account: signer) {
        let cap = treasury::remove_linear_withdraw_capability<STC>(&account);
        assert!(treasury::get_linear_withdraw_capability_withdraw(&cap) == 0, 1001);
        treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//# block --author alice


//# run --signers StarcoinAssociation
script {
    // use starcoin_framework::TreasuryScripts;

    use starcoin_framework::treasury_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer) {
        treasury_scripts::withdraw_and_split_lt_withdraw_cap<STC>(account, @alice, 100000000000000, 0);
    }
}


//# block --author alice


//# run --signers alice
script {
    use starcoin_framework::treasury;
    use starcoin_framework::stc_offer;
    use starcoin_framework::starcoin_coin::STC;

    fun redeem_offer(account: signer) {
        let cap = stc_offer::redeem<treasury::LinearWithdrawCapability<STC>>(&account, @StarcoinAssociation);
        treasury::add_linear_withdraw_capability(&account, cap);
    }
}


//# block --author alice


//# run --signers alice
script {
    use starcoin_framework::treasury_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer) {
        treasury_scripts::withdraw_token_with_linear_withdraw_capability<STC>(account);
    }
}
