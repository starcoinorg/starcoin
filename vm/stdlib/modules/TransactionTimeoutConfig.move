address 0x1 {

module TransactionTimeoutConfig {
    use 0x1::Timestamp;
    use 0x1::CoreAddresses;
    use 0x1::Config;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    struct TransactionTimeoutConfig {
        duration_seconds: u64,
    }

    public fun initialize(account: &signer, duration_seconds: u64) {
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);

        Config::publish_new_config<Self::TransactionTimeoutConfig>(
            account,
            new_transaction_timeout_config(duration_seconds)
        );
    }

    spec fun initialize {
        //aborts_if Timestamp::
        pragma verify = false;
    }

    public fun new_transaction_timeout_config(duration_seconds: u64) : TransactionTimeoutConfig {
        TransactionTimeoutConfig {duration_seconds: duration_seconds}
    }

    spec fun new_transaction_timeout_config {
        pragma verify = false;
    }

    public fun get_transaction_timeout_config(): TransactionTimeoutConfig {
        Config::get_by_address<TransactionTimeoutConfig>(CoreAddresses::GENESIS_ADDRESS())
    }

    spec fun get_transaction_timeout_config {
        pragma verify = false;
    }

    public fun duration_seconds() :u64 {
        let config = get_transaction_timeout_config();
        config.duration_seconds
    }

    spec fun duration_seconds {
        pragma verify = false;
    }
}
}