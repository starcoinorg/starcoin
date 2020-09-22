script {
    use 0x1::TransactionPublishOption;
    fun main(account: &signer) {
        TransactionPublishOption::set_open_script(account)
    }
}