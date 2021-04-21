// Test split linear mint key
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
//! block-time: 1000
//! block-number: 1

//! new-transaction
//! sender: association
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Treasury;
    use 0x1::Account;

    fun bob_take_linear_key_from_offer(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        let (token, cap2) = Treasury::split_linear_withdraw_cap(&mut cap, 47777040000000000/2);
        Offer::create(&account, cap2, {{alice}}, 0);
        Account::deposit_to_self(&account, token);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//! block-prologue
//! author: alice
//! block-time: 2000
//! block-number: 2

//! new-transaction
//! sender: alice
script {
    use 0x1::Offer;
    use 0x1::STC::STC;
    use 0x1::Treasury::{Self, LinearTimeWithdrawCapability};

    fun alice_take_linear_key_from_offer(account: signer) {
        let cap = Offer::redeem<LinearTimeWithdrawCapability<STC>>(&account, {{association}});
        assert(Treasury::get_linear_withdraw_capability_total(&cap)==47777040000000000/2, 1002);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}