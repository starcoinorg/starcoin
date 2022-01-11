//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use StarcoinFramework::Account;
    fun main(signer: signer) {
        let cap = Account::remove_signer_capability(&signer);
        let created_signer = Account::create_signer_with_cap(&cap);
        assert!(created_signer == signer, 101);
        Account::destroy_signer_cap(cap);
    }
}

// check: EXECUTED
