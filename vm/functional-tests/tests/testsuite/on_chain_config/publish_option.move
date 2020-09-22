// test the publish option can be correctly set.
// functional test use VMPublishingOption::Open as initial config.
//! account: alice

//! sender: alice
script {
    use 0x1::Signer;
    fun main(account: &signer) {
        assert(Signer::address_of(account) == {{alice}}, 8000);
    }
}
// check: "Discard(UNKNOWN_SCRIPT)"

//! new-transaction
//! sender: genesis
script {
    use 0x1::TransactionPublishOption;
    fun main(account: &signer) {
        TransactionPublishOption::set_open_script(account)
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    fun main(account: &signer) {
        use 0x1::Signer;
        assert(Signer::address_of(account) == {{alice}}, 8000);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
module Foo {
    struct T {
        version: u64,
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: genesis
script {
    use 0x1::TransactionPublishOption;
    fun main(account: &signer) {
        TransactionPublishOption::set_open_module(account, false)
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
module Foo2 {
    struct T {
        version: u64,
    }
}
// check: Discard(INVALID_MODULE_PUBLISHER)