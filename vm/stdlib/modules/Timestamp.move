address 0x1 {

module Timestamp {
    use 0x1::CoreAddresses;
    use 0x1::Signer;

    // A singleton resource holding the current Unix time in seconds
    resource struct CurrentTimeSeconds {
        seconds: u64,
    }

    // Initialize the global wall clock time resource.
    public fun initialize(account: &signer) {
        // Only callable by the Genesis address
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        // TODO: Pass the initialized value be passed in to genesis?
        let timer = CurrentTimeSeconds {seconds: 0};
        move_to<CurrentTimeSeconds>(account, timer);
    }
    spec fun initialize {
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ACCOUNT();
        aborts_if exists<CurrentTimeSeconds>(Signer::spec_address_of(account));
        ensures exists<CurrentTimeSeconds>(Signer::spec_address_of(account));
        ensures global<CurrentTimeSeconds>(Signer::spec_address_of(account)).seconds == 0;
    }

    // Update the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(account: &signer, timestamp: u64) acquires CurrentTimeSeconds {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
        let global_timer = borrow_global_mut<CurrentTimeSeconds>(CoreAddresses::GENESIS_ACCOUNT());
        //TODO should support '=' ?
        assert(global_timer.seconds <= timestamp, 5001);
        global_timer.seconds = timestamp;
    }
    spec fun update_global_time {
        aborts_if !exists<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ACCOUNT());
        ensures global<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ACCOUNT()).seconds == timestamp;
    }

    // Get the timestamp representing `now` in seconds.
    public fun now_seconds(): u64 acquires CurrentTimeSeconds {
        borrow_global<CurrentTimeSeconds>(CoreAddresses::GENESIS_ACCOUNT()).seconds
    }
    spec fun now_seconds {
        aborts_if !exists<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ACCOUNT());
        ensures result == global<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ACCOUNT()).seconds;
    }

    // Helper function to determine if the blockchain is at genesis state.
    public fun is_genesis(): bool {
        !exists<CurrentTimeSeconds>(CoreAddresses::GENESIS_ACCOUNT())
    }
    spec fun is_genesis {
        aborts_if false;
        ensures result == !exists<CurrentTimeSeconds>(CoreAddresses::SPEC_GENESIS_ACCOUNT());
    }
}
}
