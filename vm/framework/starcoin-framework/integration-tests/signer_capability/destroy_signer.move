//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::account;
    fun main(signer: signer) {
        let cap = account::remove_signer_capability(&signer);
        account::destroy_signer_cap(cap);
    }
}

// check: EXECUTED

//
//script {
//    fun main(_signer: signer) {
//    }
//}
//// check: Discard
//// check: INVALID_AUTH_KEY