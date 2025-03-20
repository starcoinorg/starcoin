//# init -n dev

//# faucet --addr alice --amount 0

// Test split linear mint key

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

//! block-prologue
//! author: alice
//! block-time: 1000
//! block-number: 1
//# block --author alice

//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::Offer;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Treasury;
    use StarcoinFramework::Account;

    fun bob_take_linear_key_from_offer(account: signer) {
        let cap = Treasury::remove_linear_withdraw_capability<STC>(&account);
        let (token, cap2) = Treasury::split_linear_withdraw_cap(&mut cap, 47777040000000000/2);
        Offer::create(&account, cap2, @alice, 0);
        Account::deposit_to_self(&account, token);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}

//! block-prologue
//! author: alice
//! block-time: 2000
//! block-number: 2
//# block --author alice

//# run --signers alice
script {
    use StarcoinFramework::Offer;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Treasury::{Self, LinearWithdrawCapability};

    fun alice_take_linear_key_from_offer(account: signer) {
        let cap = Offer::redeem<LinearWithdrawCapability<STC>>(&account, @StarcoinAssociation);
        assert!(Treasury::get_linear_withdraw_capability_total(&cap)==47777040000000000/2, 1002);
        Treasury::add_linear_withdraw_capability(&account, cap);
    }
}