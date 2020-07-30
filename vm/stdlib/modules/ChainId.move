address 0x1 {
module ChainId {
    use 0x1::CoreAddresses;
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::ErrorCode;

    resource struct ChainId {
        id: u8
    }

    /// Publish the chain ID under the genesis account
    public fun initialize(account: &signer, id: u8) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(),
            ErrorCode::ENOT_GENESIS_ACCOUNT()
        );
        move_to(account, ChainId { id });
    }

    /// Return the chain ID of this chain
    public fun get(): u8 acquires ChainId {
        borrow_global<ChainId>(CoreAddresses::GENESIS_ACCOUNT()).id
    }
}
}
