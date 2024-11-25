//# init -n dev

//# faucet --addr Genesis

//# faucet --addr alice

//# faucet --addr bob

//# run --signers bob
script {
    use starcoin_framework::stc_transaction_fee;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;

    fun pay_fees(account: signer) {
        let coin = coin::withdraw<STC>(&account, 200);
        assert!(coin::value<STC>(&coin) == 200, 8001);
        stc_transaction_fee::pay_fee<STC>(coin);
    }
}


//# run --signers Genesis
script {
    use std::signer;
    use starcoin_framework::coin;
    use starcoin_framework::starcoin_coin::{STC};
    use starcoin_framework::stc_transaction_fee;

    fun distribute_fees(account: signer) {
        let coin = stc_transaction_fee::distribute_transaction_fees<STC>(&account);
        let value = coin::value<STC>(&coin);
        assert!(value >= 200, 10000);
        coin::deposit(signer::address_of(&account), coin);
    }
}
// check: EXECUTED


//# run --signers alice
script {
    use std::signer;
    use starcoin_framework::stc_transaction_fee;
    use starcoin_framework::coin;
    use starcoin_framework::starcoin_coin::{STC};

    fun main(account: signer) {
        let coin = stc_transaction_fee::distribute_transaction_fees<STC>(&account);
        coin::deposit(signer::address_of(&account), coin);
    }
}

// check: ABORTED

