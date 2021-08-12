address 0x1 {
/// The module for StdlibUpgrade init scripts
module StdlibUpgradeScripts {

        use 0x1::CoreAddresses;
        use 0x1::STC::{Self, STC};
        use 0x1::Token::{Self, LinearTimeMintKey};
        use 0x1::TreasuryWithdrawDaoProposal;
        use 0x1::Treasury::{Self, LinearWithdrawCapability};
        use 0x1::Offer;
        use 0x1::Timestamp;
        use 0x1::Collection;
        use 0x1::Oracle;
        use 0x1::STCUSDOracle;
        use 0x1::NFT;
        use 0x1::GenesisNFT;
        spec module {
            pragma verify = false;
            pragma aborts_if_is_strict = true;
        }

        /// Stdlib upgrade script from v2 to v3
        public(script) fun upgrade_from_v2_to_v3(account: signer, total_stc_amount: u128 ) {
            CoreAddresses::assert_genesis_address(&account);

            let withdraw_cap = STC::upgrade_from_v1_to_v2(&account, total_stc_amount);

            let mint_keys = Collection::borrow_collection<LinearTimeMintKey<STC>>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());
            let mint_key = Collection::borrow(&mint_keys, 0);
            let (total, minted, start_time, period) = Token::read_linear_time_key(mint_key);
            Collection::return_collection(mint_keys);

            let now = Timestamp::now_seconds();
            let linear_withdraw_cap = Treasury::issue_linear_withdraw_capability(&mut withdraw_cap, total-minted, period - (now - start_time));
            // Lock the TreasuryWithdrawCapability to Dao
            TreasuryWithdrawDaoProposal::plugin(&account, withdraw_cap);
            // Give a LinearWithdrawCapability Offer to association, association need to take the offer, and destroy old LinearTimeMintKey.
            Offer::create(&account, linear_withdraw_cap, CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 0);
        }

        /// association account should call this script after upgrade from v2 to v3.
        public(script) fun take_linear_withdraw_capability(signer: signer){
            let offered = Offer::redeem<LinearWithdrawCapability<STC>>(&signer, CoreAddresses::GENESIS_ADDRESS());
            Treasury::add_linear_withdraw_capability(&signer, offered);
            let mint_key = Collection::take<LinearTimeMintKey<STC>>(&signer);
            Token::destroy_linear_time_key(mint_key);
        }

        public fun do_upgrade_from_v5_to_v6(sender: &signer) {
            CoreAddresses::assert_genesis_address(sender);
            Oracle::initialize(sender);
            //register oracle
            STCUSDOracle::register(sender);
            NFT::initialize(sender);
            let merkle_root = x"5969f0e8e19f8769276fb638e6060d5c02e40088f5fde70a6778dd69d659ee6d";
            let image = b"ipfs://QmSPcvcXgdtHHiVTAAarzTeubk5X3iWymPAoKBfiRFjPMY";
            GenesisNFT::initialize(sender, merkle_root, 1639u64, image);
        }

        public(script) fun upgrade_from_v5_to_v6(sender: signer) {
           Self::do_upgrade_from_v5_to_v6(&sender)
        }
}
}