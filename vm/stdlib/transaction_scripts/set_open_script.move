script {
    use 0x1::TransactionPublishOption;
    fun set_open_script(account: &signer) {
        TransactionPublishOption::set_open_script(account)
    }
}