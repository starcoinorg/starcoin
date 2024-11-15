//# init -n dev

//# faucet --addr alice


//# run --signers alice
script {
    use std::signer;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;

    fun main(account: signer) {
        let coin = coin::zero<STC>();
        coin::deposit<STC>(signer::address_of(&account), coin);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::starcoin_coin::{STC};
    use starcoin_framework::Token;
    use starcoin_framework::account;
    use starcoin_framework::signer;

    fun main(account: signer) {
        let coin = Token::zero<STC>();
        account::deposit_with_metadata<STC>(signer::address_of(&account), coin, x"");
    }
}
// check: EXECUTED
