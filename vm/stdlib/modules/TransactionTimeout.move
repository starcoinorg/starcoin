address 0x1 {

module TransactionTimeout {
  use 0x1::Signer;
  use 0x1::CoreAddresses;
  use 0x1::Timestamp;

  spec module {
      pragma verify = false;
  }

  resource struct TTL {
    // Only transactions with timestamp in between block time and block time + duration would be accepted.
    duration_seconds: u64,
  }

  public fun initialize(account: &signer) {
    // Only callable by the Genesis address
    assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);
    // Currently set to 1day.
    //TODO set by onchain config.
    move_to(account, TTL {duration_seconds: 86400});
  }
  spec fun initialize {
    aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ACCOUNT();
    aborts_if exists<TTL>(Signer::spec_address_of(account));
    ensures global<TTL>(Signer::spec_address_of(account)).duration_seconds == 86400;
  }

  public fun set_timeout(account: &signer, new_duration: u64) acquires TTL {
    // Only callable by the Genesis address
    assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

    let timeout = borrow_global_mut<TTL>(CoreAddresses::GENESIS_ACCOUNT());
    timeout.duration_seconds = new_duration;
  }
  spec fun set_timeout {
    aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ACCOUNT();
    aborts_if !exists<TTL>(CoreAddresses::SPEC_GENESIS_ACCOUNT());
    ensures global<TTL>(Signer::spec_address_of(account)).duration_seconds == new_duration;
  }

  public fun is_valid_transaction_timestamp(txn_timestamp: u64): bool acquires TTL {
    let current_block_time = Timestamp::now_seconds();
    // if now is genesis, just return true.
    // TODO: need to figure out a better way to handle this.
    if (current_block_time == 0) {
      return txn_timestamp > 0
    };
    let timeout = borrow_global<TTL>(CoreAddresses::GENESIS_ACCOUNT()).duration_seconds;
    let max_txn_time = current_block_time + timeout;
    current_block_time < txn_timestamp && txn_timestamp < max_txn_time
  }
}
}
