/// The module provides the information of current consensus strategy.
module starcoin_framework::consensus_strategy {
    use starcoin_framework::on_chain_config;
    use starcoin_framework::system_addresses;

    /// ConsensusStrategy data.
    struct ConsensusStrategy has copy, drop, store {
        /// Value of strategy
        value: u8
    }

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    /// Publish the chain ID under the genesis account
    public fun initialize(account: &signer, consensus_strategy: u8) {
        // Timestamp::assert_genesis();
        system_addresses::assert_starcoin_framework(account);
        let cap = on_chain_config::publish_new_config_with_capability<ConsensusStrategy>(
            account,
            ConsensusStrategy { value: consensus_strategy }
        );
        //destroy the cap, so ConsensusStrategy can not been change.
        on_chain_config::destroy_modify_config_capability(cap);
    }

    /// Return the consensus strategy type of this chain
    public fun get(): u8 {
        on_chain_config::get_by_address<ConsensusStrategy>(system_addresses::get_starcoin_framework()).value
    }
}
