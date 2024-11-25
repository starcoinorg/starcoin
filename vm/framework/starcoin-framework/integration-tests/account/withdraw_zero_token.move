//# init -n dev

//# faucet --addr alice


//# run --signers alice
script {
    use starcoin_framework::starcoin_coin::{STC};
    use starcoin_framework::Token;
    use starcoin_framework::account;

    fun main(account: signer) {
        let coin = coin::withdraw<STC>(&account, 0);
        Token::destroy_zero(coin);
    }
}
// check: EXECUTED