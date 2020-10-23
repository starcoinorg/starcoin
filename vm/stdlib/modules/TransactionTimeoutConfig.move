address 0x1 {

module TransactionTimeoutConfig {
    use 0x1::Timestamp;
    use 0x1::CoreAddresses;
    use 0x1::Config;
    use 0x1::Signer;

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
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        include Config::PublishNewConfigAbortsIf<TransactionTimeoutConfig>;
        include Config::PublishNewConfigEnsures<TransactionTimeoutConfig>;
    }

    public fun new_transaction_timeout_config(duration_seconds: u64) : TransactionTimeoutConfig {
        TransactionTimeoutConfig {duration_seconds: duration_seconds}
    }

    spec fun new_transaction_timeout_config {
        aborts_if false;
    }

    public fun get_transaction_timeout_config(): TransactionTimeoutConfig {
        Config::get_by_address<TransactionTimeoutConfig>(CoreAddresses::GENESIS_ADDRESS())
    }

    spec fun get_transaction_timeout_config {
        include Config::AbortsIfConfigNotExist<TransactionTimeoutConfig>{
            addr: CoreAddresses::GENESIS_ADDRESS()
        };
    }

    public fun duration_seconds() :u64 {
        let config = get_transaction_timeout_config();
        config.duration_seconds
    }

    spec fun duration_seconds {
        include Config::AbortsIfConfigNotExist<TransactionTimeoutConfig>{
            addr: CoreAddresses::GENESIS_ADDRESS()
        };
    }

    spec schema AbortsIfTxnTimeoutConfigNotExist {
        include Config::AbortsIfConfigNotExist<TransactionTimeoutConfig>{
            addr: CoreAddresses::GENESIS_ADDRESS()
        };
    }
}
}