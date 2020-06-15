address 0x1 {

module Timestamp {

    use 0x1::Signer;

    // A singleton resource holding the current Unix time in microseconds
    resource struct CurrentTimeMicroseconds {
        microseconds: u64,
    }

    // Initialize the global wall clock time resource.
    public fun initialize(account: &signer) {
        // Only callable by the Association address
        assert(Signer::address_of(account) == 0xA550C18, 1);

        // TODO: Should the initialized value be passed in to genesis?
        let timer = CurrentTimeMicroseconds {microseconds: 0};
        move_to<CurrentTimeMicroseconds>(account, timer);
    }
    spec fun initialize {
        aborts_if Signer::get_address(account) != 0xA550C18;
        aborts_if exists<CurrentTimeMicroseconds>(Signer::get_address(account));
        ensures exists<CurrentTimeMicroseconds>(Signer::get_address(account));
        ensures global<CurrentTimeMicroseconds>(Signer::get_address(account)).microseconds == 0;
    }

    // Update the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(_account: &signer, proposer: address, timestamp: u64) acquires CurrentTimeMicroseconds {
        // Can only be invoked by LibraVM privilege.
        //TODO conform addr
        //assert(Signer::address_of(account) == 0x0, 33);

        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(0xA550C18);
        if (proposer == 0x0) {
            // NIL block with null address as proposer. Timestamp must be equal.
            //TODO
            //assert(timestamp == global_timer.microseconds, 5001);
        } else {
            // Normal block. Time must advance
            //TODO
            //assert(global_timer.microseconds < timestamp, 5001);
        };
        global_timer.microseconds = timestamp;
    }
    spec fun update_global_time {
        aborts_if !exists<CurrentTimeMicroseconds>(0xA550C18);
        ensures global<CurrentTimeMicroseconds>(0xA550C18).microseconds == timestamp;
    }

    // Get the timestamp representing `now` in microseconds.
    public fun now_microseconds(): u64 acquires CurrentTimeMicroseconds {
        borrow_global<CurrentTimeMicroseconds>(0xA550C18).microseconds
    }
    spec fun now_microseconds {
        aborts_if !exists<CurrentTimeMicroseconds>(0xA550C18);
        ensures result == global<CurrentTimeMicroseconds>(0xA550C18).microseconds;
    }

    // Helper function to determine if the blockchain is at genesis state.
    public fun is_genesis(): bool {
        !exists<Self::CurrentTimeMicroseconds>(0xA550C18)
    }
    spec fun is_genesis {
        aborts_if false;
        ensures result == !exists<CurrentTimeMicroseconds>(0xA550C18);
    }
}
}
