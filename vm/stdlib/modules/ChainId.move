address 0x1 {
/// The module provides chain id information.
module ChainId {
    use 0x1::CoreAddresses;
    use 0x1::Timestamp;
    use 0x1::Signer;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// chain id data structure.
    struct ChainId has key {
        /// real id.
        id: u8
    }

    /// Publish the chain ID under the genesis account
    public fun initialize(account: &signer, id: u8) {
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);
        move_to(account, ChainId { id });
    }

    spec initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<ChainId>(Signer::spec_address_of(account));
        ensures exists<ChainId>(Signer::spec_address_of(account));
    }

    /// Return the chain ID of this chain
    public fun get(): u8 acquires ChainId {
        borrow_global<ChainId>(CoreAddresses::GENESIS_ADDRESS()).id
    }

    spec get {
        aborts_if !exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
}
}
