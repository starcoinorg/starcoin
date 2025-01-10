//# init -n dev

//# faucet --addr alice --amount 100

//# faucet --addr Genesis

//# run --signers Genesis
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::coin;
    use starcoin_framework::asset_mapping;

    fun test_asset_mapping_assign_to_account_with_proof(framework: signer) {
        assert!(coin::balance<STC>(@alice) == 100, 10001);
        asset_mapping::assign_to_account(&framework, @alice, b"0x1::STC::STC", 100);
        assert!(coin::balance<STC>(@alice) == 200, 10002);
    }
}
// check: EXECUTED