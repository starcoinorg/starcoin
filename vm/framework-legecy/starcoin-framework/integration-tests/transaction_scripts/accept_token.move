//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use StarcoinFramework::Account;
    use StarcoinFramework::DummyToken::DummyToken;

    fun main(account: signer) {
        Account::accept_token<DummyToken>(account);
    }
}
