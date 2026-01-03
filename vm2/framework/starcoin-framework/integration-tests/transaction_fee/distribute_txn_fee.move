//# init -n dev

//# faucet --addr Genesis

//# faucet --addr alice

//# faucet --addr bob

//# run --signers bob
script {
    use std::option;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::coin::paired_metadata;
    use starcoin_framework::fungible_asset;
    use starcoin_framework::transaction_fee;
    use starcoin_framework::starcoin_coin::STC;

    fun pay_fees(bob: &signer) {
        let stc_metadata = option::destroy_some(paired_metadata<STC>());
        let fa = primary_fungible_store::withdraw(bob, stc_metadata, 200);
        assert!(fungible_asset::amount(&fa) == 200, 8001);
        transaction_fee::pay_fee(bob, fa);
    }
}

//# run --signers Genesis
script {
    use starcoin_framework::stc_block;

    fun block_epilogue(account: signer) {
        stc_block::block_epilogue(&account, 200);
    }
}
// check: EXECUTED


//# run --signers Genesis
script {
    use std::signer;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::fungible_asset;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::transaction_fee;

    fun distribute_fees(genesis: &signer) {
        let fa = transaction_fee::withdraw_account_transaction_fees(genesis, starcoin_coin::get_stc_fa_metadata());
        assert!(fungible_asset::amount(&fa) >= 200, 10000);
        primary_fungible_store::deposit(signer::address_of(genesis), fa);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use std::signer;
    use starcoin_framework::transaction_fee::pay_fee;
    use starcoin_framework::starcoin_coin;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::transaction_fee;

    fun alice_withdraw_transaction_fee(alice: &signer) {
        pay_fee(alice, primary_fungible_store::withdraw(alice, starcoin_coin::get_stc_fa_metadata(), 100));
        let fa = transaction_fee::withdraw_account_transaction_fees(alice, starcoin_coin::get_stc_fa_metadata());
        primary_fungible_store::deposit(signer::address_of(alice), fa);
    }
}
// check: ABORTED
