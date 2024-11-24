//# init -n dev

//# faucet --addr alice

//# faucet --addr bob


//# run --signers bob
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;

    fun main(account: signer) {
        coin::register<STC>(&account);
    }
}
