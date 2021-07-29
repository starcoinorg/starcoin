//! new-transaction
//! sender: genesis
script {
    use 0x1::Account;
    fun main(signer: signer) {
        let cap = Account::remove_signer_capability(&signer);
        Account::destroy_signer_cap(cap);
    }
}

// check: EXECUTED

////! new-transaction
////! sender: genesis
//script {
//    fun main(_signer: signer) {
//    }
//}
//// check: Discard
//// check: INVALID_AUTH_KEY