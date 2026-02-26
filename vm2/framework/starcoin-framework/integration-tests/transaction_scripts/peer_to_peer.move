//# init -n dev


//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr carol


//# run --signers alice --args @bob --args x"" --args 100u128
script {
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer, payee: address, payee_auth_key: vector<u8>, amount: u128) {
        transfer_scripts::peer_to_peer<STC>(&account, payee, payee_auth_key, amount);
    }
}

//# run --signers alice --args @bob --args 100u128
script {
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer, payee: address, amount: u128) {
        transfer_scripts::peer_to_peer_v2<STC>(&account, payee, amount);
    }
}

//# run --signers alice
script {
    use starcoin_framework::coin;
    use starcoin_framework::transfer_scripts;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer) {
        let bob_before = coin::balance<STC>(@bob);
        let carol_before = coin::balance<STC>(@carol);

        transfer_scripts::batch_peer_to_peer_v2<STC>(
            &account,
            vector[@bob, @carol],
            vector[100u128, 200u128],
        );

        assert!(coin::balance<STC>(@bob) == bob_before + 100, 1001);
        assert!(coin::balance<STC>(@carol) == carol_before + 200, 1002);
    }
}
