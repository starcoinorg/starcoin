address 0x1 {
module ChainId {
    use 0x1::CoreAddresses;
    use 0x1::Timestamp;
    use 0x1::Signer;

    resource struct ChainId {
        id: u8
    }

    const ENOT_GENESIS: u64 = 0;
    const ENOT_GENESIS_ACCOUNT: u64 = 1;

    /// Publish the chain ID under the genesis account
    public fun initialize(account: &signer, id: u8) {
        assert(Timestamp::is_genesis(), ENOT_GENESIS);
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(),
            ENOT_GENESIS_ACCOUNT
        );

        move_to(account, ChainId { id })
    }

    /// Return the chain ID of this Libra instance
    public fun get(): u8 acquires ChainId {
        borrow_global<ChainId>(CoreAddresses::GENESIS_ACCOUNT()).id
    }
}
}
