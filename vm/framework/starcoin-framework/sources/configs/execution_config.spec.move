spec starcoin_framework::execution_config {
    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict;
    }

    /// Ensure the caller is admin
    /// When setting now time must be later than last_reconfiguration_time.
    spec set(account: &signer, config: vector<u8>) {
        use starcoin_framework::timestamp;
        use std::signer;
        use std::features;
        use starcoin_framework::transaction_fee;
        use starcoin_framework::chain_status;
        use starcoin_framework::stake;
        use starcoin_framework::staking_config;
        use starcoin_framework::starcoin_coin;

        // TODO: set because of timeout (property proved)
        pragma verify_duration_estimate = 600;
        let addr = signer::address_of(account);
        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockStarcoinSupply;
        requires chain_status::is_genesis();
        requires exists<stake::ValidatorFees>(@starcoin_framework);
        requires exists<staking_config::StakingRewardsConfig>(@starcoin_framework);
        requires len(config) > 0;
        include features::spec_periodical_reward_rate_decrease_enabled() ==> staking_config::StakingRewardsConfigEnabledRequirement;
        include starcoin_coin::ExistsAptosCoin;
        requires system_addresses::is_starcoin_framework_address(addr);
        requires timestamp::spec_now_microseconds() >= reconfiguration::last_reconfiguration_time();

        ensures exists<ExecutionConfig>(@starcoin_framework);
    }

    spec set_for_next_epoch(account: &signer, config: vector<u8>) {
        include config_buffer::SetForNextEpochAbortsIf;
    }

    spec on_new_epoch(framework: &signer) {
        requires @starcoin_framework == std::signer::address_of(framework);
        include config_buffer::OnNewEpochRequirement<ExecutionConfig>;
        aborts_if false;
    }
}
