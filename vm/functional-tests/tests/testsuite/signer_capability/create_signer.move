//! new-transaction
//! sender: genesis
script {
    use 0x1::Account;
    fun main(signer: signer) {
        let cap = Account::remove_signer_capability(&signer);
        let created_signer = Account::create_signer_with_cap(&cap);
        assert(created_signer == signer, 101);
        Account::destroy_signer_cap(cap);
    }
}

// check: EXECUTED
