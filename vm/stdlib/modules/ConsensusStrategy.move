address 0x1 {
module ConsensusStrategy {
    use 0x1::CoreAddresses;
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::Config;

    struct ConsensusStrategy {
        value: u8
    }

    spec module {
        pragma verify;
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

    spec fun initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<Config::Config<ConsensusStrategy>>(Signer::spec_address_of(account));
        aborts_if exists<Config::ModifyConfigCapabilityHolder<ConsensusStrategy>>(Signer::spec_address_of(account));
        ensures exists<Config::Config<ConsensusStrategy>>(Signer::spec_address_of(account));
    }

    /// Return the consensus strategy type of this chain
    public fun get(): u8 {
        Config::get_by_address<ConsensusStrategy>(CoreAddresses::GENESIS_ADDRESS()).value
    }

    spec fun get {
        aborts_if !exists<Config::Config<ConsensusStrategy>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
}
}
