/// A module used to check expiration time of transactions.
module starcoin_framework::stc_transaction_timeout {

    use starcoin_framework::stc_transaction_timeout_config;
    use starcoin_framework::stc_block;
    use starcoin_framework::timestamp;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    spec fun spec_is_valid_transaction_timestamp(txn_timestamp: u64): bool {
        if (stc_block::get_current_block_number() == 0) {
            txn_timestamp > timestamp::now_seconds()
        } else {
            timestamp::now_seconds() < txn_timestamp && txn_timestamp <
                (timestamp::now_seconds() + stc_transaction_timeout_config::duration_seconds())
        }
    }

    /// Check whether the given timestamp is valid for transactions.
    public fun is_valid_transaction_timestamp(txn_timestamp: u64): bool {
        let current_block_time = timestamp::now_seconds();
        let block_number = stc_block::get_current_block_number();
        // before first block, just require txn_timestamp > genesis timestamp.
        if (block_number == 0) {
            return txn_timestamp > current_block_time
        };
        let timeout = stc_transaction_timeout_config::duration_seconds();
        let max_txn_time = current_block_time + timeout;
        current_block_time < txn_timestamp && txn_timestamp < max_txn_time
    }

    spec is_valid_transaction_timestamp {
        use starcoin_framework::stc_block;
        use starcoin_framework::timestamp;
        use starcoin_framework::system_addresses;
        use starcoin_framework::on_chain_config;

        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if !exists<stc_block::BlockMetadata>(system_addresses::get_starcoin_framework());
        // include Timestamp::AbortsIfTimestampNotExists;

        aborts_if stc_block::get_current_block_number() != 0
            && timestamp::now_seconds() + stc_transaction_timeout_config::duration_seconds() > max_u64();
        aborts_if stc_block::get_current_block_number() != 0
            && !exists<on_chain_config::Config<stc_transaction_timeout_config::TransactionTimeoutConfig>>(
            system_addresses::get_starcoin_framework()
        );
    }

    spec schema AbortsIfTimestampNotValid {
        use starcoin_framework::stc_block;
        use starcoin_framework::timestamp;
        use starcoin_framework::stc_transaction_timeout_config;
        use starcoin_framework::on_chain_config;
        use starcoin_framework::system_addresses;

        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if !exists<stc_block::BlockMetadata>(system_addresses::get_starcoin_framework());
        // include timestamp::AbortsIfTimestampNotExists;

        aborts_if stc_block::get_current_block_number() != 0 && timestamp::now_seconds(
        ) + stc_transaction_timeout_config::duration_seconds() > max_u64();
        aborts_if stc_block::get_current_block_number(
        ) != 0 && !exists<on_chain_config::Config<stc_transaction_timeout_config::TransactionTimeoutConfig>>(
            system_addresses::get_starcoin_framework()
        );
    }
}
