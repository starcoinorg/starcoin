script {
    use StarcoinFramework::Debug;
    use StarcoinFramework::Signer;

    fun test_debug(account: signer){
        Debug::print(&Signer::address_of(&account));
    }
}