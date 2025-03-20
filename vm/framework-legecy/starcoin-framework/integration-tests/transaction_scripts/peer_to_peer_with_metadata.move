//# init -n dev


//# faucet --addr alice

//# faucet --addr bob

//# run --signers alice --args @bob --args x"" --args 100u128 --args x""
script {
    use StarcoinFramework::TransferScripts;
    use StarcoinFramework::STC::STC;

    fun main(account: signer, payee: address, payee_auth_key: vector<u8>, amount: u128, metadata: vector<u8>) {
        TransferScripts::peer_to_peer_with_metadata<STC>(account, payee, payee_auth_key, amount, metadata);
    }
}

//# run --signers alice --args @bob --args 100u128 --args x""
script {
    use StarcoinFramework::TransferScripts;
    use StarcoinFramework::STC::STC;

    fun main(account: signer, payee: address, amount: u128, metadata: vector<u8>) {
        TransferScripts::peer_to_peer_with_metadata_v2<STC>(account, payee, amount, metadata);
    }
}
