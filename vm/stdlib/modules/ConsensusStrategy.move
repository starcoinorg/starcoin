address 0x1 {
module ConsensusStrategy {
    use 0x1::CoreAddresses;
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::ErrorCode;
    use 0x1::Config;

    struct ConsensusStrategy {
        value: u8
    }

    /// Publish the chain ID under the genesis account
    public fun initialize(account: &signer, consensus_strategy: u8) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(),
            ErrorCode::ENOT_GENESIS_ACCOUNT()
        );
        let cap = Config::publish_new_config_with_capability<ConsensusStrategy>(
            account,
            ConsensusStrategy { value:consensus_strategy }
        );
        //destory the cap, so ConsensusStrategy can not been change.
        Config::destory_modify_config_capability(cap);
    }

    /// Return the consensus strategy type of this chain
    public fun get(): u8 {
        Config::get_by_address<ConsensusStrategy>(CoreAddresses::GENESIS_ACCOUNT()).value
    }
}
}
