address StarcoinFramework {
/// The module provides the information of current consensus strategy.    
module ConsensusStrategy {
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Config;

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
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);
        let cap = Config::publish_new_config_with_capability<ConsensusStrategy>(
            account,
            ConsensusStrategy { value:consensus_strategy }
        );
        //destroy the cap, so ConsensusStrategy can not been change.
        Config::destroy_modify_config_capability(cap);
    }

    spec initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<Config::Config<ConsensusStrategy>>(Signer::address_of(account));
        aborts_if exists<Config::ModifyConfigCapabilityHolder<ConsensusStrategy>>(Signer::address_of(account));
        ensures exists<Config::Config<ConsensusStrategy>>(Signer::address_of(account));
    }

    /// Return the consensus strategy type of this chain
    public fun get(): u8 {
        Config::get_by_address<ConsensusStrategy>(CoreAddresses::GENESIS_ADDRESS()).value
    }

    spec get {
        aborts_if !exists<Config::Config<ConsensusStrategy>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
}
}
