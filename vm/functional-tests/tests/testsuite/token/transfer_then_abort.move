//! account: alice, 10000 0x1::STC::STC
//! account: bob, 10000 0x1::STC::STC

//! sender: alice

script {
    use 0x1::Account;

    fun main(account: &signer) {
        Account::pay_from<0x1::STC::STC>(account, {{bob}}, 10);
        abort 41
    }
}
// txn is kept
// check: ABORTED
// check: 41

//! new-transaction
//! sender: bob

script {
    use 0x1::Account;

    fun main() {
        // check the state is unchanged
        assert(Account::balance<0x1::STC::STC>({{bob}}) == 10000, 42);
    }
}

// check: EXECUTED