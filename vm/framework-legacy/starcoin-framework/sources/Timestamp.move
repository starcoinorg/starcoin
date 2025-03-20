address StarcoinFramework {
/// The module implements onchain timestamp oracle.
/// Timestamp is updated on each block. It always steps forward, and never come backward.
module Timestamp {
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }


    // A singleton resource holding the current Unix time in milliseconds
    struct CurrentTimeMilliseconds has key {
        milliseconds: u64,
    }

    /// A singleton resource used to determine whether time has started. This
    /// is called at the end of genesis.
    struct TimeHasStarted has key {}

    /// Conversion factor between seconds and milliseconds
    const MILLI_CONVERSION_FACTOR: u64 = 1000;

    const ENOT_GENESIS: u64 = 12;
    const EINVALID_TIMESTAMP: u64 = 14;
    const ENOT_INITIALIZED: u64 = 101;
    // Initialize the global wall clock time resource.
    public fun initialize(account: &signer, genesis_timestamp: u64) {
        // Only callable by the Genesis address
        CoreAddresses::assert_genesis_address(account);
        let milli_timer = CurrentTimeMilliseconds {milliseconds: genesis_timestamp};
        move_to<CurrentTimeMilliseconds>(account, milli_timer);
    }
    spec initialize {
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<CurrentTimeMilliseconds>(Signer::address_of(account));
        ensures exists<CurrentTimeMilliseconds>(Signer::address_of(account));
    }

    // Update the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(account: &signer, timestamp: u64) acquires CurrentTimeMilliseconds {
        CoreAddresses::assert_genesis_address(account);
        //Do not update time before time start.
        let global_milli_timer = borrow_global_mut<CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS());
        assert!(timestamp > global_milli_timer.milliseconds, Errors::invalid_argument(EINVALID_TIMESTAMP));
        global_milli_timer.milliseconds = timestamp;
    }
    spec update_global_time {
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if timestamp <= global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds;
        ensures global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds == timestamp;
    }

    // Get the timestamp representing `now` in seconds.
    public fun now_seconds(): u64 acquires CurrentTimeMilliseconds {
        now_milliseconds() / MILLI_CONVERSION_FACTOR
    }
    spec now_seconds {
        aborts_if !exists<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures result == now_milliseconds() / MILLI_CONVERSION_FACTOR;
    }
    spec fun spec_now_seconds(): u64 {
        global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds / MILLI_CONVERSION_FACTOR
    }

    // Get the timestamp representing `now` in milliseconds.
    public fun now_milliseconds(): u64 acquires CurrentTimeMilliseconds {
        borrow_global<CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS()).milliseconds
    }

    spec now_milliseconds {
        aborts_if !exists<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures result == global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds;
    }

    spec fun spec_now_millseconds(): u64 {
        global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds
    }

    /// Marks that time has started and genesis has finished. This can only be called from genesis.
    public fun set_time_has_started(account: &signer) {
        CoreAddresses::assert_genesis_address(account);

        // Current time must have been initialized.
        assert!(
            exists<CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS()),
            Errors::invalid_state(ENOT_INITIALIZED)
        );
        move_to(account, TimeHasStarted{});
    }

    spec set_time_has_started {
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<CurrentTimeMilliseconds>(Signer::address_of(account));
        aborts_if exists<TimeHasStarted>(Signer::address_of(account));
        ensures exists<TimeHasStarted>(Signer::address_of(account));
    }

    /// Helper function to determine if the blockchain is in genesis state.
    public fun is_genesis(): bool {
        !exists<TimeHasStarted>(CoreAddresses::GENESIS_ADDRESS())
    }

    spec is_genesis {
        aborts_if false;
        ensures result == !exists<TimeHasStarted>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Helper function to assert genesis state.
    public fun assert_genesis() {
        assert!(is_genesis(), Errors::invalid_state(ENOT_GENESIS));
    }
    spec assert_genesis {
        pragma opaque = true;
        include AbortsIfNotGenesis;
    }

    /// Helper schema to specify that a function aborts if not in genesis.
    spec schema AbortsIfNotGenesis {
        aborts_if !is_genesis();
    }

    spec schema AbortsIfTimestampNotExists {
        aborts_if !exists<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
}
}
