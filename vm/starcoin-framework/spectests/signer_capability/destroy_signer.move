//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use StarcoinFramework::Account;
    fun main(signer: signer) {
        let cap = Account::remove_signer_capability(&signer);
        Account::destroy_signer_cap(cap);
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