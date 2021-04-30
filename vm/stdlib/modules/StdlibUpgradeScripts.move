address 0x1 {
/// The module for StdlibUpgrade init scripts
module StdlibUpgradeScripts {

        use 0x1::CoreAddresses;
        use 0x1::STC;
        use 0x1::TreasuryWithdrawDaoProposal;
        /// Stdlib upgrade script from v2 to v3
        public(script) fun upgrade_from_v2_to_v3(account: signer, total_stc_amount: u128) {
            CoreAddresses::assert_genesis_address(&account);
            let withdraw_cap = STC::upgrade_from_v1_to_v2(&account, total_stc_amount);
            // Lock the TreasuryWithdrawCapability to Dao
            TreasuryWithdrawDaoProposal::plugin(&account, withdraw_cap);
        }
}
}