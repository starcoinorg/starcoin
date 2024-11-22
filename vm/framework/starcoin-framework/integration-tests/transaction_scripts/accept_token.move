//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# publish
module alice::dummy_coin {

    /// The DummyToken type.
    struct DummyCoin has copy, drop, store {}

    public fun init(account: &signer) {

    }
}
//# run --signers bob
script {
    use starcoin_framework::account;
    use alice::dummy_coin::dummy;

    fun main(account: signer) {
        account::accept_token<DummyToken>(account);
    }
}
