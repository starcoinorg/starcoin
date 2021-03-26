//! account: alice

//! sender: alice
script {
    use 0x1::Account;
    use 0x1::DummyToken::DummyToken;

    fun main(account: signer) {
        Account::accept_token<DummyToken>(account);
    }
}

// check: gas_used
// check: 37198
// check: EXECUTED