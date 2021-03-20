//! account: alice, 100000 0x1::STC::STC
//! account: bob

//! sender: alice
//! args: {{bob}}, x"", 100u128
script {
    use 0x1::TransferScripts;
    use 0x1::STC::STC;

    fun main(account: &signer, payee: address, payee_auth_key: vector<u8>, amount: u128) {
        TransferScripts::peer_to_peer<STC>(account, payee, payee_auth_key, amount);
    }
}
// check: gas_used
// check: 128424
// check: "Keep(EXECUTED)"

