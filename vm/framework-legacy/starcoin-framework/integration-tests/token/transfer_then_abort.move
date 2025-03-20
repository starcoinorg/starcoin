//# init -n dev

//# faucet --addr alice --amount 1000000

//# faucet --addr bob --amount 1000000

//# run --signers alice
script {
    use StarcoinFramework::Account;

    fun main(account: signer) {
        Account::pay_from<StarcoinFramework::STC::STC>(&account, @bob, 10);
        abort 41
    }
}
// txn is kept
// check: ABORTED
// check: 41

//# run --signers bob
script {
    use StarcoinFramework::Account;

    fun main() {
        // check the state is unchanged
        assert!(Account::balance<StarcoinFramework::STC::STC>(@bob) == 1000000, 42);
    }
}

// check: EXECUTED