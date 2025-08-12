/// Onchain configuration for timeout setting of transaction.
module starcoin_framework::stc_transaction_timeout_config {

    use starcoin_framework::system_addresses;
    use starcoin_framework::on_chain_config;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    /// config structs.
    struct TransactionTimeoutConfig has copy, drop, store {
        /// timeout in second.
        duration_seconds: u64,
    }

    /// Initialize function. Should only be called in genesis.
    public fun initialize(account: &signer, duration_seconds: u64) {
        // Timestamp::assert_genesis();
        system_addresses::assert_starcoin_framework(account);

        on_chain_config::publish_new_config<Self::TransactionTimeoutConfig>(
            account,
            new_transaction_timeout_config(duration_seconds)
        );
    }

    spec initialize {
        use starcoin_framework::on_chain_config;
        use starcoin_framework::signer;

        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        include on_chain_config::PublishNewConfigAbortsIf<TransactionTimeoutConfig>;
        include on_chain_config::PublishNewConfigEnsures<TransactionTimeoutConfig>;
    }

    /// Create a new timeout config used in dao proposal.
    public fun new_transaction_timeout_config(duration_seconds: u64): TransactionTimeoutConfig {
        TransactionTimeoutConfig { duration_seconds: duration_seconds }
    }

    spec new_transaction_timeout_config {
        aborts_if false;
    }

    /// Get current timeout config.
    public fun get_transaction_timeout_config(): TransactionTimeoutConfig {
        on_chain_config::get_by_address<TransactionTimeoutConfig>(system_addresses::get_starcoin_framework())
    }

    spec get_transaction_timeout_config {
        use starcoin_framework::system_addresses;

        include on_chain_config::AbortsIfConfigNotExist<TransactionTimeoutConfig> {
            addr: system_addresses::get_starcoin_framework()
        };
    }

    /// Get current txn timeout in seconds.
    public fun duration_seconds(): u64 {
        let config = get_transaction_timeout_config();
        config.duration_seconds
    }

    spec duration_seconds {
        include on_chain_config::AbortsIfConfigNotExist<TransactionTimeoutConfig> {
            addr: system_addresses::get_starcoin_framework()
        };
    }

    spec schema AbortsIfTxnTimeoutConfigNotExist {
        include on_chain_config::AbortsIfConfigNotExist<TransactionTimeoutConfig> {
            addr: system_addresses::get_starcoin_framework()
        };
    }
}