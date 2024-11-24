//# init -n dev

//# faucet --addr alice --amount 0

// Test split linear mint key

//# run --signers StarcoinAssociation
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::treasury;

    fun mint(account: signer) {
        let cap = treasury::remove_linear_withdraw_capability<STC>(&account);
        assert!(treasury::get_linear_withdraw_capability_withdraw(&cap) == 0, 1001);
        treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1
//# block --author alice

//# run --signers StarcoinAssociation
script {
    use std::signer;
    use starcoin_framework::stc_offer;
    use starcoin_framework::coin;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::treasury;

    fun bob_take_linear_key_from_offer(account: signer) {
        let cap = treasury::remove_linear_withdraw_capability<STC>(&account);
        let (token, cap2) = treasury::split_linear_withdraw_cap(&mut cap, 47777040000000000 / 2);
        stc_offer::create(&account, cap2, @alice, 0);
        coin::deposit(signer::address_of(&account), token);
        treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//! block-prologue
//! author: alice
//! block-time: 2000
//! block-number: 2
//# block --author alice

//# run --signers alice
script {
    use starcoin_framework::stc_offer;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::treasury::{Self, LinearWithdrawCapability};

    fun alice_take_linear_key_from_offer(account: signer) {
        let cap =
            stc_offer::redeem<LinearWithdrawCapability<STC>>(&account, @StarcoinAssociation);
        assert!(treasury::get_linear_withdraw_capability_total(&cap) == 47777040000000000 / 2, 1002);
        treasury::add_linear_withdraw_capability(&account, cap);
    }
}