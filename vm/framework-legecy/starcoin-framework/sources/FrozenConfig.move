/// The module provide configuration for frozen configuration.
module StarcoinFramework::FrozenConfig {
    use StarcoinFramework::ACL;
    use StarcoinFramework::Config;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Vector;

    friend StarcoinFramework::FrozenConfigStrategy;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    struct FrozenConfig has copy, drop, store {
        frozen_global_txn: bool,
        frozen_account_list: ACL::ACL
    }

    const ERR_CONFIG_NOT_EXISTS: u64 = 101;

    public fun initialize(account: &signer, frozen_account_list: ACL::ACL) {
        if (Config::config_exist_by_address<Self::FrozenConfig>(Signer::address_of(account))) {
            return
        };

        Config::publish_new_config<Self::FrozenConfig>(
            account,
            FrozenConfig {
                frozen_global_txn: false,
                frozen_account_list,
            }
        );
    }

    public fun set_account_list(account: &signer, frozen_account_list: ACL::ACL) {
        let addr = Signer::address_of(account);
        assert!(
            Config::config_exist_by_address<FrozenConfig>(addr),
            Errors::invalid_state(ERR_CONFIG_NOT_EXISTS)
        );

        let config= Config::get_by_address<FrozenConfig>(addr);
        Config::set<FrozenConfig>(
            account,
            FrozenConfig {
                frozen_global_txn: config.frozen_global_txn,
                frozen_account_list,
            }
        );
    }

    public fun set_global_frozen(account: &signer, frozen: bool) {
        let addr = Signer::address_of(account);
        assert!(
            Config::config_exist_by_address<FrozenConfig>(addr),
            Errors::invalid_state(ERR_CONFIG_NOT_EXISTS)
        );

        let config = Config::get_by_address<FrozenConfig>(addr);
        Config::set<FrozenConfig>(
            account,
            FrozenConfig {
                frozen_global_txn: frozen,
                frozen_account_list: config.frozen_account_list,
            }
        );
    }

    spec initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::ASSOCIATION_ROOT_ADDRESS();
        aborts_if Vector::length(frozen_account_list.list) == 0;
        aborts_if exists<Config::Config<FrozenConfig>>(Signer::address_of(account));
        include Config::PublishNewConfigAbortsIf<FrozenConfig>;
        include Config::PublishNewConfigEnsures<FrozenConfig>;
    }

    /// Get frozen configuration.
    public fun get_frozen_config(account: address): FrozenConfig {
        Config::get_by_address<FrozenConfig>(account)
    }

    spec get_frozen_config {
        aborts_if !exists<Config::Config<FrozenConfig>>(account);
    }

    public fun get_frozen_account_list(account: address): ACL::ACL {
        let config = get_frozen_config(account);
        config.frozen_account_list
    }

    /// Get global frozen
    public fun get_frozen_global(account: address): bool {
        let config = get_frozen_config(account);
        config.frozen_global_txn
    }


}