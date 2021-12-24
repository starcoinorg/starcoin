//# init -n dev

//# faucet --addr alice --amount 0

//# run --signers StarcoinAssociation
script {
    use Std::STC::STC;
    use Std::Treasury;
    //use Std::Debug;

    fun mint(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        assert!(Treasury::get_linear_withdraw_capability_withdraw(&cap) == 0, 1001);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//# block --author alice


//# run --signers StarcoinAssociation
script {
    use Std::TreasuryScripts;
    use Std::STC::STC;

    fun main(account: signer) {
        TreasuryScripts::withdraw_and_split_lt_withdraw_cap<STC>(account, @alice, 100000000000000, 0);
    }
}


//# block --author alice


//# run --signers alice
script {
    use Std::Offer;
    use Std::STC::STC;
    use Std::Treasury;

    fun redeem_offer(account: signer) {
        let cap = Offer::redeem<Treasury::LinearWithdrawCapability<STC>>(&account, @StarcoinAssociation);
        Treasury::add_linear_withdraw_capability(&account,cap);
    }
}


//# block --author alice


//# run --signers alice

script {
    use Std::TreasuryScripts;
    use Std::STC::STC;

    fun main(account: signer) {
        TreasuryScripts::withdraw_token_with_linear_withdraw_capability<STC>(account);
    }
}
