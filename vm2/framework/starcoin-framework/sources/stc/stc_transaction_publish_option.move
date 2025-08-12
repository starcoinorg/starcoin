/// `TransactionPublishOption` provide an option to limit:
/// - whether user can use script or publish custom modules on chain.
module starcoin_framework::transaction_publish_option {

    use std::error;
    use std::signer;
    use starcoin_std::debug;
    use starcoin_framework::on_chain_config;
    use starcoin_framework::system_addresses;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict = true;
    }
    spec fun spec_is_script_allowed(addr: address): bool {
        let publish_option = on_chain_config::get_by_address<TransactionPublishOption>(addr);
        publish_option.script_allowed
    }

    spec fun spec_is_module_allowed(addr: address): bool {
        let publish_option = on_chain_config::get_by_address<TransactionPublishOption>(addr);
        publish_option.module_publishing_allowed
    }

    const SCRIPT_HASH_LENGTH: u64 = 32;

    const EPROLOGUE_ACCOUNT_DOES_NOT_EXIST: u64 = 0;
    const EINVALID_ARGUMENT: u64 = 18;
    /// The script hash has an invalid length
    const EINVALID_SCRIPT_HASH: u64 = 1001;
    /// The script hash already exists in the allowlist
    const EALLOWLIST_ALREADY_CONTAINS_SCRIPT: u64 = 1002;

    /// Defines and holds the publishing policies for the VM. There are three possible configurations:
    /// 1.  !script_allowed && !module_publishing_allowed No module publishing, only script function in module are allowed.
    /// 2.  script_allowed && !module_publishing_allowed No module publishing, custom scripts are allowed.
    /// 3.  script_allowed && module_publishing_allowed Both module publishing and custom scripts are allowed.
    /// We represent these as the following resource.
    struct TransactionPublishOption has copy, drop, store {
        // Anyone can use script if this flag is set to true.
        script_allowed: bool,
        // Anyone can publish new module if this flag is set to true.
        module_publishing_allowed: bool,
    }

    /// Module initialization.
    public fun initialize(
        account: &signer,
        script_allowed: bool,
        module_publishing_allowed: bool,
    ) {
        debug::print(&std::string::utf8(b"stc_transaction_publish_option::initialize | entered "));

        // timestamp::assert_genesis();
        assert!(
            signer::address_of(account) == system_addresses::get_starcoin_framework(),
            error::not_found(EPROLOGUE_ACCOUNT_DOES_NOT_EXIST),
        );
        let transaction_publish_option = Self::new_transaction_publish_option(
            script_allowed,
            module_publishing_allowed
        );
        on_chain_config::publish_new_config(
            account,
            transaction_publish_option,
        );
        debug::print(&std::string::utf8(b"stc_transaction_publish_option::initialize | exited "));
    }

    spec initialize {
        // aborts_if !Timestamp::is_genesis();
        use std::signer;

        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        include on_chain_config::PublishNewConfigAbortsIf<TransactionPublishOption>;
        include on_chain_config::PublishNewConfigEnsures<TransactionPublishOption>;
    }

    /// Create a new option. Mainly used in DAO.
    public fun new_transaction_publish_option(
        script_allowed: bool,
        module_publishing_allowed: bool,
    ): TransactionPublishOption {
        TransactionPublishOption { script_allowed, module_publishing_allowed }
    }

    spec new_transaction_publish_option {
        aborts_if false;
    }

    /// Check if sender can execute script with
    public fun is_script_allowed(account: address): bool {
        let publish_option = on_chain_config::get_by_address<TransactionPublishOption>(account);
        publish_option.script_allowed
    }

    spec is_script_allowed {
        include on_chain_config::AbortsIfConfigNotExist<TransactionPublishOption> {
            addr: account
        };
    }

    /// Check if a sender can publish a module
    public fun is_module_allowed(account: address): bool {
        let publish_option = on_chain_config::get_by_address<TransactionPublishOption>(account);
        publish_option.module_publishing_allowed
    }

    spec is_module_allowed {
        include on_chain_config::AbortsIfConfigNotExist<TransactionPublishOption> {
            addr: account
        };
    }

    spec schema AbortsIfTxnPublishOptionNotExist {
        include on_chain_config::AbortsIfConfigNotExist<TransactionPublishOption> {
            addr: system_addresses::get_starcoin_framework()
        };
    }

    spec schema AbortsIfTxnPublishOptionNotExistWithBool {
        is_script_or_package: bool;
        aborts_if is_script_or_package && !exists<on_chain_config::Config<TransactionPublishOption>>(
            system_addresses::get_starcoin_framework()
        );
    }
}