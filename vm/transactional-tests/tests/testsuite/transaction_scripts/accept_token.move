//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
    use Std::Account;
    use Std::DummyToken::DummyToken;

    fun main(account: signer) {
        Account::accept_token<DummyToken>(account);
    }
}
