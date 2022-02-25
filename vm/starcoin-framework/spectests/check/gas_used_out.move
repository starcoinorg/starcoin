//# init -n dev

//# faucet --addr alice --amount 1000

//# faucet --addr bob --amount 1000

//# faucet --addr default

//# run --signers alice --gas-budget 700

// when gas used out, the txn is kept, the state is unchanged except balance is set to 0.

script {
    use StarcoinFramework::Account;

    fun main(account: signer) {
        Account::pay_from<StarcoinFramework::STC::STC>(&account, @bob, 10);
        Account::pay_from<StarcoinFramework::STC::STC>(&account, @bob, 10);
        Account::pay_from<StarcoinFramework::STC::STC>(&account, @bob, 10);
        Account::pay_from<StarcoinFramework::STC::STC>(&account, @bob, 10);
        Account::pay_from<StarcoinFramework::STC::STC>(&account, @bob, 10);
        Account::pay_from<StarcoinFramework::STC::STC>(&account, @bob, 10);
        Account::pay_from<StarcoinFramework::STC::STC>(&account, @bob, 10);
        // gas used out
    }
}
// check: EXECUTION_FAILURE
// check: OUT_OF_GAS
// check: gas_used
// check: 700


//# run --signers default
script {
    use StarcoinFramework::Account;

    fun main() {
        // check the state is unchanged
        assert!(Account::balance<StarcoinFramework::STC::STC>(@bob) == 1000, 42);
        assert!(Account::balance<StarcoinFramework::STC::STC>(@alice) == 300, 43);
    }
}

// check: EXECUTED