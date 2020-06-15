address 0x1 {

module TransactionTimeout {
  use 0x1::Signer;

  use 0x1::Timestamp;

  resource struct TTL {
    // Only transactions with timestamp in between block time and block time + duration would be accepted.
    duration_microseconds: u64,
  }

  public fun initialize(association: &signer) {
    // Only callable by the Association address
    assert(Signer::address_of(association) == 0xA550C18, 1);
    // Currently set to 1day.
    move_to(association, TTL {duration_microseconds: 86400000000});
  }
  spec fun initialize {
    aborts_if Signer::get_address(association) != 0xA550C18;
    aborts_if exists<TTL>(Signer::get_address(association));
    ensures global<TTL>(Signer::get_address(association)).duration_microseconds == 86400000000;
  }

  public fun set_timeout(association: &signer, new_duration: u64) acquires TTL {
    // Only callable by the Association address
    assert(Signer::address_of(association) == 0xA550C18, 1);

    let timeout = borrow_global_mut<TTL>(0xA550C18);
    timeout.duration_microseconds = new_duration;
  }
  spec fun set_timeout {
    aborts_if Signer::get_address(association) != 0xA550C18;
    aborts_if !exists<TTL>(0xA550C18);
    ensures global<TTL>(Signer::get_address(association)).duration_microseconds == new_duration;
  }

  public fun is_valid_transaction_timestamp(timestamp: u64): bool acquires TTL {
    // Reject timestamp greater than u64::MAX / 1_000_000;
    if(timestamp > 9223372036854) {
      return false
    };

    let current_block_time = Timestamp::now_microseconds();
    let timeout = borrow_global<TTL>(0xA550C18).duration_microseconds;
    let _max_txn_time = current_block_time + timeout;

    let txn_time_microseconds = timestamp * 1000000;
    // TODO: Add Timestamp::is_before_exclusive(&txn_time_microseconds, &max_txn_time)
    //       This is causing flaky test right now. The reason is that we will use this logic for AC, where its wall
    //       clock time might be out of sync with the real block time stored in StateStore.
    //       See details in issue #2346.
    current_block_time < txn_time_microseconds
  }
  spec fun is_valid_transaction_timestamp {
    aborts_if timestamp <= 9223372036854 && !exists<Timestamp::CurrentTimeMicroseconds>(0xA550C18);
    aborts_if timestamp <= 9223372036854 && !exists<TTL>(0xA550C18);
    aborts_if timestamp <= 9223372036854 && global<Timestamp::CurrentTimeMicroseconds>(0xA550C18).microseconds + global<TTL>(0xA550C18).duration_microseconds > max_u64();
    aborts_if timestamp <= 9223372036854 && timestamp * 1000000 > max_u64();
    ensures timestamp > 9223372036854 ==> result == false;
    ensures timestamp <= 9223372036854 ==> result == (global<Timestamp::CurrentTimeMicroseconds>(0xA550C18).microseconds < timestamp * 1000000);
  }
}
}
