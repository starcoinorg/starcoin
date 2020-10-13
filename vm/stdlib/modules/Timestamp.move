address 0x1 {

module Timestamp {
    use 0x1::CoreAddresses;
    use 0x1::Signer;
    use 0x1::ErrorCode;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }


    // A singleton resource holding the current Unix time in milliseconds
    resource struct CurrentTimeMilliseconds {
        milliseconds: u64,
    }

    /// Conversion factor between seconds and milliseconds
    const MILLI_CONVERSION_FACTOR: u64 = 1000;

    /// A singleton resource used to determine whether time has started. This
    /// is called at the end of genesis.
    resource struct TimeHasStarted {}

    // Initialize the global wall clock time resource.
    public fun initialize(account: &signer, genesis_timestamp: u64) {
        // Only callable by the Genesis address
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        let milli_timer = CurrentTimeMilliseconds {milliseconds: genesis_timestamp};
        move_to<CurrentTimeMilliseconds>(account, milli_timer);
    }
    spec fun initialize {
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<CurrentTimeMilliseconds>(Signer::spec_address_of(account));
        ensures exists<CurrentTimeMilliseconds>(Signer::spec_address_of(account));
    }

    // Update the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(account: &signer, timestamp: u64) acquires CurrentTimeMilliseconds {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        //Do not update time before time start.
        let global_milli_timer = borrow_global_mut<CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS());
        assert(timestamp > global_milli_timer.milliseconds, ErrorCode::EINVALID_TIMESTAMP());
        global_milli_timer.milliseconds = timestamp;
    }
    spec fun update_global_time {
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if timestamp < global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds;
        ensures global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds == timestamp;
    }

    // Get the timestamp representing `now` in seconds.
    public fun now_seconds(): u64 acquires CurrentTimeMilliseconds {
        now_milliseconds() / MILLI_CONVERSION_FACTOR
    }
    spec fun now_seconds {
        aborts_if !exists<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures result == now_milliseconds() / MILLI_CONVERSION_FACTOR;
    }
    spec define spec_now_seconds(): u64 {
        global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds / MILLI_CONVERSION_FACTOR
    }

    // Get the timestamp representing `now` in milliseconds.
    public fun now_milliseconds(): u64 acquires CurrentTimeMilliseconds {
        borrow_global<CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS()).milliseconds
    }

    spec fun now_milliseconds {
        aborts_if !exists<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures result == global<CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).milliseconds;
    }

    /// Marks that time has started and genesis has finished. This can only be called from genesis.
    public fun set_time_has_started(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());

        // Current time must have been initialized.
        assert(
            exists<CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS()),
            1
        );
        move_to(account, TimeHasStarted{});
    }

    spec fun set_time_has_started {
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<CurrentTimeMilliseconds>(Signer::spec_address_of(account));
        aborts_if exists<TimeHasStarted>(Signer::spec_address_of(account));
        ensures exists<TimeHasStarted>(Signer::spec_address_of(account));
    }

    /// Helper function to determine if the blockchain is in genesis state.
    public fun is_genesis(): bool {
        !exists<TimeHasStarted>(CoreAddresses::GENESIS_ADDRESS())
    }

    spec fun is_genesis {
        aborts_if false;
        ensures result == !exists<TimeHasStarted>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
}
}
