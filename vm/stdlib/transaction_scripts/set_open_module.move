script {
    use 0x1::TransactionPublishOption;
    fun set_open_module(account: &signer, open_module: bool) {
        TransactionPublishOption::set_open_module(account, open_module)
    }
}
