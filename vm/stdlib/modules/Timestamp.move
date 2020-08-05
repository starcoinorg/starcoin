address 0x1 {

module Timestamp {
    use 0x1::CoreAddresses;
    use 0x1::Signer;
    use 0x1::ErrorCode;

    // A singleton resource holding the current Unix time in seconds
    resource struct CurrentTimeSeconds {
        seconds: u64,
    }

    /// A singleton resource used to determine whether time has started. This
    /// is called at the end of genesis.
    resource struct TimeHasStarted {}

    // Initialize the global wall clock time resource.
    public fun initialize(account: &signer, genesis_timestamp: u64) {
        // Only callable by the Genesis address
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        let timer = CurrentTimeSeconds {seconds: genesis_timestamp};
        move_to<CurrentTimeSeconds>(account, timer);
    }
    spec fun initialize {
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<CurrentTimeSeconds>(Signer::spec_address_of(account));
        ensures exists<CurrentTimeSeconds>(Signer::spec_address_of(account));
        ensures global<CurrentTimeSeconds>(Signer::spec_address_of(account)).seconds == 0;
    }

    // Update the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(account: &signer, timestamp: u64) acquires CurrentTimeSeconds {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        //Do not update time before time start.
        let global_timer = borrow_global_mut<CurrentTimeSeconds>(CoreAddresses::GENESIS_ADDRESS());
        assert(timestamp > global_timer.seconds, ErrorCode::EINVALID_TIMESTAMP());
        global_timer.seconds = timestamp;
    }
    spec fun update_global_time {
        aborts_if !exists<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures global<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).seconds == timestamp;
    }

    // Get the timestamp representing `now` in seconds.
    public fun now_seconds(): u64 acquires CurrentTimeSeconds {
        borrow_global<CurrentTimeSeconds>(CoreAddresses::GENESIS_ADDRESS()).seconds
    }
    spec fun now_seconds {
        aborts_if !exists<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        ensures result == global<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ADDRESS()).seconds;
    }

    /// Marks that time has started and genesis has finished. This can only be called from genesis.
    public fun set_time_has_started(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());

        // Current time must have been initialized.
        assert(
            exists<CurrentTimeSeconds>(CoreAddresses::GENESIS_ADDRESS()),
            1
        );
        move_to(account, TimeHasStarted{});
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
