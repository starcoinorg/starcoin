/// This module keeps a global wall clock that stores the current Unix time in microseconds.
/// It interacts with the other modules in the following ways:
/// * genesis: to initialize the timestamp
/// * block: to reach consensus on the global wall clock time
module starcoin_framework::timestamp {
    use starcoin_framework::system_addresses;
    use std::error;

    friend starcoin_framework::genesis;
    friend starcoin_framework::stc_genesis;

    /// A singleton resource holding the current Unix time in microseconds
    struct CurrentTimeMicroseconds has key {
        microseconds: u64,
    }

    /// Conversion factor between seconds and microseconds
    const MICRO_CONVERSION_FACTOR: u64 = 1000000;

    /// Conversion factor between seconds and microseconds
    const MILLI_CONVERSION_FACTOR: u64 = 1000;

    /// The blockchain is not in an operating state yet
    const ENOT_OPERATING: u64 = 1;
    /// An invalid timestamp was provided
    const EINVALID_TIMESTAMP: u64 = 2;

    /// Marks that time has started. This can only be called from genesis and with the starcoin framework account.
    public(friend) fun set_time_has_started(starcoin_framework: &signer) {
        system_addresses::assert_starcoin_framework(starcoin_framework);
        let timer = CurrentTimeMicroseconds { microseconds: 0 };
        move_to(starcoin_framework, timer);
    }

    /// Updates the wall clock time by consensus. Requires VM privilege and will be invoked during block prologue.
    public fun update_global_time(
        account: &signer,
        proposer: address,
        timestamp: u64
    ) acquires CurrentTimeMicroseconds {
        // Can only be invoked by StarcoinVM signer.
        system_addresses::assert_vm(account);

        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(@starcoin_framework);
        let now = global_timer.microseconds;
        if (proposer == @vm_reserved) {
            // NIL block with null address as proposer. Timestamp must be equal.
            assert!(now == timestamp, error::invalid_argument(EINVALID_TIMESTAMP));
        } else {
            // Normal block. Time must advance
            assert!(now < timestamp, error::invalid_argument(EINVALID_TIMESTAMP));
            global_timer.microseconds = timestamp;
        };
    }

    #[test_only]
    public fun set_time_has_started_for_testing(account: &signer) {
        if (!exists<CurrentTimeMicroseconds>(@starcoin_framework)) {
            set_time_has_started(account);
        };
    }

    #[view]
    /// Gets the current time in microseconds.
    public fun now_microseconds(): u64 acquires CurrentTimeMicroseconds {
        borrow_global<CurrentTimeMicroseconds>(@starcoin_framework).microseconds
    }

    #[view]
    /// Gets the current time in milliseconds.
    public fun now_milliseconds(): u64 acquires CurrentTimeMicroseconds {
        now_microseconds() / MILLI_CONVERSION_FACTOR
    }

    #[view]
    /// Gets the current time in seconds.
    public fun now_seconds(): u64 acquires CurrentTimeMicroseconds {
        now_microseconds() / MICRO_CONVERSION_FACTOR
    }


    #[test_only]
    public fun update_global_time_for_test(timestamp_microsecs: u64) acquires CurrentTimeMicroseconds {
        let global_timer = borrow_global_mut<CurrentTimeMicroseconds>(@starcoin_framework);
        let now = global_timer.microseconds;
        assert!(now < timestamp_microsecs, error::invalid_argument(EINVALID_TIMESTAMP));
        global_timer.microseconds = timestamp_microsecs;
    }

    #[test_only]
    public fun update_global_time_for_test_secs(timestamp_seconds: u64) acquires CurrentTimeMicroseconds {
        update_global_time_for_test(timestamp_seconds * MICRO_CONVERSION_FACTOR);
    }

    #[test_only]
    public fun fast_forward_seconds(timestamp_seconds: u64) acquires CurrentTimeMicroseconds {
        update_global_time_for_test_secs(now_seconds() + timestamp_seconds);
    }
}
