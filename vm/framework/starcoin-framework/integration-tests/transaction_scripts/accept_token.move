//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::account;
    use starcoin_framework::DummyToken::DummyToken;

    fun main(account: signer) {
        account::accept_token<DummyToken>(account);
    }
}
