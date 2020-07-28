script {
    use 0x1::Debug;
    use 0x1::Signer;

    fun test_debug(account: &signer){
        Debug::print(&Signer::address_of(account));
    }
}