//# init -n dev

//# faucet --addr alice --amount 0

//# run --signers StarcoinAssociation
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::treasury;
    //use starcoin_framework::Debug;

    fun mint(account: signer) {
        let cap = treasury::remove_linear_withdraw_capability<STC>(&account);
        assert!(treasury::get_linear_withdraw_capability_total(&cap) == 477770400000000000, 1000);
        assert!(treasury::get_linear_withdraw_capability_withdraw(&cap) == 0, 1001);
        assert!(treasury::get_linear_withdraw_capability_start_time(&cap) == 0, 1002);
        starcoin_framework::debug::print(&treasury::get_linear_withdraw_capability_period(&cap));
        assert!(treasury::get_linear_withdraw_capability_period(&cap) == 86400, 1003);
        treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//# block --author alice --timestamp 3600000

//# run --signers StarcoinAssociation
script {
    use std::signer;
    use starcoin_std::debug;
    use starcoin_framework::coin;
    use starcoin_framework::treasury;
    use starcoin_framework::starcoin_coin::STC;

    fun mint(account: signer) {
        let linear_cap = treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = treasury::withdraw_with_linear_capability(&mut linear_cap);
        debug::print(&coin::value(&token));
        assert!(coin::value(&token) == 19907100000000000, 1004);
        treasury::add_linear_withdraw_capability(&account, linear_cap);
        coin::deposit(signer::address_of(&account), token);
    }
}


//# block --author alice --timestamp 7200000

//# run --signers StarcoinAssociation
script {
    use std::signer;
    use starcoin_std::debug;

    use starcoin_framework::coin;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::treasury;

    fun mint(account: signer) {
        let linear_cap = treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = treasury::withdraw_with_linear_capability(&mut linear_cap);
        debug::print(&coin::value(&token));
        assert!(coin::value(&token) == 19907100000000000, 1005);
        treasury::add_linear_withdraw_capability(&account, linear_cap);
        coin::deposit(signer::address_of(&account), token);
    }
}


//# block --author alice --timestamp 94608000000

//# run --signers StarcoinAssociation
script {
    use std::signer;
    use starcoin_framework::coin;
    use starcoin_std::debug;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::treasury;

    fun mint(account: signer) {
        let cap = treasury::remove_linear_withdraw_capability<STC>(&account);
        let token = treasury::withdraw_with_linear_capability(&mut cap);
        debug::print(&coin::value(&token));
        assert!(coin::value(&token) == (477770400000000000 - 19907100000000000 * 2), 1006);
        coin::deposit(signer::address_of(&account), token);
        assert!(
            treasury::get_linear_withdraw_capability_withdraw(&cap) == treasury::get_linear_withdraw_capability_total(
                &cap
            ),
            1007
        );
        treasury::destroy_linear_withdraw_capability(cap);
    }
}