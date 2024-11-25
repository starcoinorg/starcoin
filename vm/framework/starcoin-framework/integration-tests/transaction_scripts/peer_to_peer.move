//# init -n dev


//# faucet --addr alice

//# faucet --addr bob


//# run --signers alice --args @bob --args x"" --args 100u128
script {
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer, payee: address, payee_auth_key: vector<u8>, amount: u128) {
        transfer_scripts::peer_to_peer<STC>(account, payee, payee_auth_key, amount);
    }
}

//# run --signers alice --args @bob --args 100u128
script {
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer, payee: address, amount: u128) {
        transfer_scripts::peer_to_peer_v2<STC>(account, payee, amount);
    }
}
