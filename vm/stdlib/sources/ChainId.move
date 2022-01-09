address StarcoinFramework {
/// The module provides chain id information.
module ChainId {
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Signer;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// chain id data structure.
    struct ChainId has key {
        /// real id.
        id: u8
    }

    const MAIN_CHAIN_ID: u8 = 1;
    const BARNARD_CHAIN_ID: u8 = 251;
    const PROXIMA_CHAIN_ID: u8 = 252;
    const HALLEY_CHAIN_ID: u8 = 253;
    const DEV_CHAIN_ID: u8 = 254;
    const TEST_CHAIN_ID: u8 = 255;

    /// Publish the chain ID under the genesis account
    public fun initialize(account: &signer, id: u8) {
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);
        move_to(account, ChainId { id });
    }

    spec initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<ChainId>(Signer::address_of(account));
        ensures exists<ChainId>(Signer::address_of(account));
    }

    /// Return the chain ID of this chain
    public fun get(): u8 acquires ChainId {
        borrow_global<ChainId>(CoreAddresses::GENESIS_ADDRESS()).id
    }

    public fun is_dev(): bool acquires ChainId {
        get() == DEV_CHAIN_ID
    }
    public fun is_test(): bool acquires ChainId {
        get() == TEST_CHAIN_ID
    }
    public fun is_halley(): bool acquires ChainId {
        get() == HALLEY_CHAIN_ID
    }
    public fun is_proxima(): bool acquires ChainId {
        get() == PROXIMA_CHAIN_ID
    }
    public fun is_barnard(): bool acquires ChainId {
        get() == BARNARD_CHAIN_ID
    }
    public fun is_main(): bool acquires ChainId {
        get() == MAIN_CHAIN_ID
    }

    spec is_dev {
        aborts_if !exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
    spec is_test {
        aborts_if !exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
    spec is_halley {
        aborts_if !exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
    spec is_proxima {
        aborts_if !exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
    spec is_barnard {
        aborts_if !exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
    spec is_main {
        aborts_if !exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    spec get {
        aborts_if !exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures exists<ChainId>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
}
}
