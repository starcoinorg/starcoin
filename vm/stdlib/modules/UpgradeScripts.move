address 0x1 {
/// The module for UpgradeScripts
module UpgradeScripts {

        use 0x1::CoreAddresses;
        use 0x1::STC;
        use 0x1::PackageTxnManager;
        use 0x1::TreasuryWithdrawDaoProposal;

        public(script) fun upgrade_from_v1_to_v2(account: signer, total_stc_amount: u128) {
            CoreAddresses::assert_genesis_address(&account);
            let withdraw_cap = STC::upgrade_from_v1_to_v2(&account, total_stc_amount);
            // Lock the TreasuryWithdrawCapability to Dao
            TreasuryWithdrawDaoProposal::plugin(&account, withdraw_cap);
            PackageTxnManager::convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2(&account, 0x1);
        }
}
}