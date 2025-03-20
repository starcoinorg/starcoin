//# init -n dev

//# faucet --addr alice --amount 1000000000000000

//# faucet --addr bob --amount 1000000000000000

//# faucet --addr Genesis --amount 1000000000000000


//# run --signers Genesis
script {
    use StarcoinFramework::FrozenConfigStrategy;

    fun initialize_with_starcoin_association(sender: signer) {
        FrozenConfigStrategy::initialize(&sender, 0, 0, 0, 0);
    }
}
// check: Executed


//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::FrozenConfigStrategy;

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
    use StarcoinFramework::FrozenConfigStrategy;

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
    use StarcoinFramework::FrozenConfigStrategy;

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
    use StarcoinFramework::FrozenConfigStrategy;

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

////////////////////////////////////////////////////////////

//# block --author 0x1 --timestamp 1000000000

//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::FrozenConfigStrategy;
    use StarcoinFramework::Block;

    fun get_block_number_and_update_burn_block_number(account: signer) {
        assert!(Block::get_current_block_number() == 2, 10020);
        FrozenConfigStrategy::update_burn_block_number(account, 3);
    }
}
// check: EXECUTED

//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::FrozenConfigStrategy;

    fun add_alice_to_frozen_list(account: signer) {
        FrozenConfigStrategy::add_account(account, @alice);
    }
}
// check: EXECUTED


//# run --signers bob
script {
    use StarcoinFramework::FrozenConfigStrategy;

    fun try_to_burn_frozen_list_got_error(_account: signer) {
        FrozenConfigStrategy::do_burn_frozen();
    }
}
// check: FrozenConfigStrategy: "abort_code": "27137"


//# block --author 0x1 --timestamp 1000001000

//# run --signers bob
script {
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Account;
    use StarcoinFramework::FrozenConfigStrategy;

    fun do_burn_by_bob(_account: signer) {
        assert!(Account::balance<STC>(@alice) > 0, 10030);

        FrozenConfigStrategy::do_burn_frozen();

        assert!(Account::balance<STC>(@alice) == 0, 10031);
    }
}
// check: EXECUTED