// when gas used out, the txn is kept, the state is unchanged except balance is set to 0.

//! account: alice, 1000 0x1::STC::STC
//! account: bob, 1000 0x1::STC::STC

//! sender: alice
//! gas-price: 1
//! max-gas: 1000

script {
    use 0x1::Account;

    fun main(account: &signer) {
        Account::pay_from<0x1::STC::STC>(account, {{bob}}, 10);
        Account::pay_from<0x1::STC::STC>(account, {{bob}}, 10);
        Account::pay_from<0x1::STC::STC>(account, {{bob}}, 10);
        Account::pay_from<0x1::STC::STC>(account, {{bob}}, 10);
        Account::pay_from<0x1::STC::STC>(account, {{bob}}, 10);
        // gas used out
    }
}
// check: EXECUTION_FAILURE
// check: OUT_OF_GAS
// check: gas_used
// check: 1000

//! new-transaction
//! sender: bob

script {
    use 0x1::Account;

    fun main() {
        // check the state is unchanged
        assert(Account::balance<0x1::STC::STC>({{bob}}) == 1000, 42);
        assert(Account::balance<0x1::STC::STC>({{alice}}) == 0, 43);
    }
}

// check: EXECUTED