//# init -n dev

//# faucet --addr alice --amount 1000

//# faucet --addr bob --amount 1000

//# faucet --addr default

//# run --signers alice --gas-budget 700

// when gas used out, the txn is kept, the state is unchanged except balance is set to 0.

script {
    use starcoin_framework::coin;

    fun main(account: signer) {
        coin::transfer<starcoin_framework::starcoin_coin::STC>(&account, @bob, 10);
        coin::transfer<starcoin_framework::starcoin_coin::STC>(&account, @bob, 10);
        coin::transfer<starcoin_framework::starcoin_coin::STC>(&account, @bob, 10);
        coin::transfer<starcoin_framework::starcoin_coin::STC>(&account, @bob, 10);
        coin::transfer<starcoin_framework::starcoin_coin::STC>(&account, @bob, 10);
        coin::transfer<starcoin_framework::starcoin_coin::STC>(&account, @bob, 10);
        coin::transfer<starcoin_framework::starcoin_coin::STC>(&account, @bob, 10);
        // gas used out
    }
}
// check: EXECUTION_FAILURE
// check: OUT_OF_GAS
// check: gas_used
// check: 700


//# run --signers default
script {
    use starcoin_framework::coin;

    fun main() {
        // check the state is unchanged
        assert!(coin::balance<starcoin_framework::starcoin_coin::STC>(@bob) == 1000, 42);
        assert!(coin::balance<starcoin_framework::starcoin_coin::STC>(@alice) == 300, 43);
    }
}

// check: EXECUTED