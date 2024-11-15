//# init -n dev

//# faucet --addr alice --amount 10000000000


//# run --signers alice
script {
    use starcoin_framework::Debug;
    use starcoin_framework::signer;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::account;
    use starcoin_framework::StdlibUpgradeScripts;

    fun burn_illegal_token_for_upgrade(sender: signer) {
        let sender_addr = signer::address_of(&sender);
        Debug::print(&sender_addr);
        let balance = coin::balance<STC>(sender_addr);
        Debug::print(&balance);
        StdlibUpgradeScripts::burn_illegal_token(sender, 9999000000);
        let balance_1 = coin::balance<STC>(sender_addr);
        Debug::print(&balance_1);
        assert!(balance_1 <= 1000000, 10010);
    }
}
// check: EXECUTED