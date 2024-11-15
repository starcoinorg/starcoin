//# init -n dev

//# faucet --addr alice --amount 1000000

//# faucet --addr bob --amount 1000000

//# run --signers alice
script {
    use starcoin_framework::account;

    fun main(account: signer) {
        coin::transfer<starcoin_framework::starcoin_coin::STC>(&account, @bob, 10);
        abort 41
    }
}
// txn is kept
// check: ABORTED
// check: 41

//# run --signers bob
script {
    use starcoin_framework::account;

    fun main() {
        // check the state is unchanged
        assert!(account::balance<starcoin_framework::starcoin_coin::STC>(@bob) == 1000000, 42);
    }
}

// check: EXECUTED