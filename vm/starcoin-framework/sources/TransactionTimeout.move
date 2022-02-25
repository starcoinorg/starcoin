address StarcoinFramework {
/// A module used to check expiration time of transactions.
module TransactionTimeout {
  use StarcoinFramework::CoreAddresses;
  use StarcoinFramework::Timestamp;
  use StarcoinFramework::Block;
  use StarcoinFramework::TransactionTimeoutConfig;
  use StarcoinFramework::Config;

  spec module {
      pragma verify;
      pragma aborts_if_is_strict;

  }

  spec fun spec_is_valid_transaction_timestamp(txn_timestamp: u64):bool {
    if (Block::get_current_block_number() == 0) {
      txn_timestamp > Timestamp::now_seconds()
    } else {
        Timestamp::now_seconds() < txn_timestamp && txn_timestamp <
        (Timestamp::now_seconds() + TransactionTimeoutConfig::duration_seconds())
    }
  }

  /// Check whether the given timestamp is valid for transactions.
  public fun is_valid_transaction_timestamp(txn_timestamp: u64): bool {
    let current_block_time = Timestamp::now_seconds();
    let block_number = Block::get_current_block_number();
    // before first block, just require txn_timestamp > genesis timestamp.
    if (block_number == 0) {
      return txn_timestamp > current_block_time
    };
    let timeout = TransactionTimeoutConfig::duration_seconds();
    let max_txn_time = current_block_time + timeout;
    current_block_time < txn_timestamp && txn_timestamp < max_txn_time
  }
  spec is_valid_transaction_timestamp {
    aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    aborts_if !exists<Block::BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    include Timestamp::AbortsIfTimestampNotExists;
    aborts_if Block::get_current_block_number() != 0 && Timestamp::now_seconds() + TransactionTimeoutConfig::duration_seconds() > max_u64();
    aborts_if Block::get_current_block_number() != 0 && !exists<Config::Config<TransactionTimeoutConfig::TransactionTimeoutConfig>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
  }

    spec schema AbortsIfTimestampNotValid {
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if !exists<Block::BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        include Timestamp::AbortsIfTimestampNotExists;
        aborts_if Block::get_current_block_number() != 0 && Timestamp::now_seconds() + TransactionTimeoutConfig::duration_seconds() > max_u64();
        aborts_if Block::get_current_block_number() != 0 && !exists<Config::Config<TransactionTimeoutConfig::TransactionTimeoutConfig>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }
}
}
