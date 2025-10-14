//# init -n dev

//# faucet --addr alice --amount 100000000

//# faucet --addr bob --amount 100000000

//# faucet --addr default

//# run --signers alice --gas-budget 2000000

// when gas used out, the txn is kept, the state is unchanged except balance is set to 0.
script {
    use std::option;
    use std::string::utf8;
    use starcoin_std::debug;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::coin;

    fun main(alice: &signer) {
        let stc_metadata = option::destroy_some(coin::paired_metadata<STC>());
        primary_fungible_store::transfer(alice, stc_metadata, @bob, 10);
        primary_fungible_store::transfer(alice, stc_metadata, @bob, 10);
        primary_fungible_store::transfer(alice, stc_metadata, @bob, 10);
        primary_fungible_store::transfer(alice, stc_metadata, @bob, 10);
        primary_fungible_store::transfer(alice, stc_metadata, @bob, 10);

        // primary_fungible_store::transfer(alice, stc_metadata, @bob, 10);
        debug::print(&utf8(b"bob-1"));
        debug::print(&primary_fungible_store::balance(@bob, stc_metadata));

        assert!(primary_fungible_store::balance(@bob, stc_metadata) == 100000050, 1);
        // gas used out, transfer above has reverted
    }
}
// check: EXECUTION_FAILURE
// check: OUT_OF_GAS
// check: gas_used
// check: 700


//# run --signers default
script {
    use std::option;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::coin;

    fun main() {
        let stc_metadata = option::destroy_some(coin::paired_metadata<STC>());
        assert!(primary_fungible_store::balance(@alice, stc_metadata) <= 100000000, 42);
        assert!(primary_fungible_store::balance(@bob, stc_metadata) == 100000000, 43);
    }
}

// check: EXECUTED