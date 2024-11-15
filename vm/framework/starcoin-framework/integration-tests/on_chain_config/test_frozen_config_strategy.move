//# init -n dev

//# faucet --addr alice --amount 1000000000000000

//# faucet --addr bob --amount 1000000000000000

//# faucet --addr Genesis --amount 1000000000000000



// //# run --signers StarcoinAssociation
// script {
//     use starcoin_framework::FrozenConfigStrategy;
//
//     fun initialize_with_starcoin_association(sender: signer) {
//         FrozenConfigStrategy::do_initialize(&sender);
//     }
// }
// // check: Executed


//# run --signers StarcoinAssociation
script {
    use starcoin_framework::FrozenConfigStrategy;

    fun set_global_frozen_true(sender: signer) {
        FrozenConfigStrategy::set_global_frozen(sender, true);
        assert!(FrozenConfigStrategy::has_frozen_global(@alice), 10010);
    }
}
// check: Executed


//# run --signers alice
script {
    fun execution_failed_set_global_frozen(_account: signer) {}
}
// check: "status_code": "40", "status_code_name": "SEND_TXN_GLOBAL_FROZEN"

//# run --signers StarcoinAssociation
script {
    use starcoin_framework::FrozenConfigStrategy;

    fun set_global_frozen_true(sender: signer) {
        FrozenConfigStrategy::set_global_frozen(sender, false);
        assert!(!FrozenConfigStrategy::has_frozen_global(@alice), 10020);
    }
}
// check: Executed

//# run --signers alice
script {
    fun execution_succeed_after_open_frozen(_account: signer) {}
}
// check: EXECUTED

//# run --signers StarcoinAssociation
script {
    use starcoin_framework::FrozenConfigStrategy;

    fun add_alice_to_black_list(sender: signer) {
        FrozenConfigStrategy::add_account(sender, @alice);
    }
}
// check: EXECUTED

//# run --signers alice
script {
    fun execution_failed_for_alice_add_to_frozen(_account: signer) {}
}
// check: "status_code": "18", "status_code_name": "SENDING_ACCOUNT_FROZEN"

//# run --signers StarcoinAssociation
script {
    use starcoin_framework::FrozenConfigStrategy;

    fun add_alice_to_black_list(sender: signer) {
        FrozenConfigStrategy::remove_account(sender, @alice);
    }
}
// check: EXECUTED

//# run --signers alice
script {
    fun execution_succeed_for_alice_remove_from_frozen(_account: signer) {}
}
// check: EXECUTED

//# run --signers Genesis
script {
    use starcoin_framework::starcoin_coin::{Self, STC};
    use starcoin_framework::account;

    fun burn_illegal_tokens(sender: signer) {
        let illegal_token = account::withdraw_illegal_token<STC>(&sender, @alice, 0);
        STC::burn(illegal_token);
        assert!(coin::balance<STC>(@alice) == 0, 10030);
    }
}
// check: EXECUTED

//# run --signers bob
script {
    use starcoin_framework::starcoin_coin::{Self, STC};
    use starcoin_framework::account;

    fun bob_call_withdraw_illegal_token_failed(sender: signer) {
        let illegal_token = account::withdraw_illegal_token<STC>(&sender, @alice, 0);
        STC::burn(illegal_token);
    }
}
// check: "abort_code": "2818"