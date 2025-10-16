//# init -n dev

//# faucet --addr alice --amount 1000000

//# faucet --addr bob --amount 1000000

//# run --signers alice
script {
    use std::option::destroy_some;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::primary_fungible_store;

    fun main(account: signer) {
        primary_fungible_store::transfer(&account, destroy_some(coin::paired_metadata<STC>()), @bob, 10);
        abort 41
    }
}
// txn is kept
// check: ABORTED
// check: 41

//# run --signers bob
script {
    use std::option::destroy_some;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::primary_fungible_store;
    use starcoin_framework::coin;

    fun main() {
        // check the state is unchanged
        assert!(primary_fungible_store::balance(@bob, destroy_some(coin::paired_metadata<STC>())) == 1000000, 42);
    }
}
// check: EXECUTED