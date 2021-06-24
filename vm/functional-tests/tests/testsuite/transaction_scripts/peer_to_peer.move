//! account: alice, 100000 0x1::STC::STC
//! account: bob

//! sender: alice
//! args: {{bob}}, x"", 100u128
script {
    use 0x1::TransferScripts;
    use 0x1::STC::STC;

    fun main(account: signer, payee: address, payee_auth_key: vector<u8>, amount: u128) {
        TransferScripts::peer_to_peer<STC>(account, payee, payee_auth_key, amount);
    }
}
// check: gas_used
// check: 132084
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
//! args: {{bob}}, 100u128
script {
    use 0x1::TransferScripts;
    use 0x1::STC::STC;

    fun main(account: signer, payee: address, amount: u128) {
        TransferScripts::peer_to_peer_v2<STC>(account, payee, amount);
    }
}
// check: gas_used
// check: 127845
// check: "Keep(EXECUTED)"
