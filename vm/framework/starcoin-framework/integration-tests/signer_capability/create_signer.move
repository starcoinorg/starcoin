//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::account;
    fun main(signer: signer) {
        let cap = account::remove_signer_capability(&signer);
        let created_signer = account::create_signer_with_cap(&cap);
        assert!(created_signer == signer, 101);
        account::destroy_signer_cap(cap);
    }
}

// check: EXECUTED
