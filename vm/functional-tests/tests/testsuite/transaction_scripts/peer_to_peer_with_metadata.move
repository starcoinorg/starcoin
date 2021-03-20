//! account: alice, 100000 0x1::STC::STC
//! account: bob

//! sender: alice
//! args: {{bob}}, x"", 100u128, x""
script {
    use 0x1::TransferScripts;
    use 0x1::STC::STC;

    fun main(account: &signer, payee: address, payee_auth_key: vector<u8>, amount: u128, metadata: vector<u8>) {
        TransferScripts::peer_to_peer_with_metadata<STC>(account, payee, payee_auth_key, amount, metadata);
    }
}
// check: gas_used
// check: 125375
// check: "Keep(EXECUTED)"

