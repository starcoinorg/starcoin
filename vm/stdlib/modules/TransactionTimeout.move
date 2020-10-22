address 0x1 {

module TransactionTimeout {
  use 0x1::CoreAddresses;
  use 0x1::Timestamp;
  use 0x1::Block;
  use 0x1::TransactionTimeoutConfig;

  spec module {
      pragma verify;
      pragma aborts_if_is_strict;
  }

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
  spec fun is_valid_transaction_timestamp {
    aborts_if Timestamp::now_seconds() + TransactionTimeoutConfig::duration_seconds() > max_u64();
    aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    aborts_if !exists<Block::BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
  }

    spec schema AbortsIfTimestampNotValid {
        aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if !exists<Block::BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        include Timestamp::AbortsIfTimestampNotExists;
        //aborts_if !exists<TransactionTimeoutConfig::TransactionTimeoutConfig>(CoreAddresses::GENESIS_ADDRESS());
        //aborts_if Timestamp::now_seconds() + TransactionTimeoutConfig::duration_seconds() > max_u64();
    }
}
}
