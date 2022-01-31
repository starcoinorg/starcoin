address StarcoinFramework {
/// `TransactionPublishOption` provide an option to limit:
/// - whether user can use script or publish custom modules on chain.
module TransactionPublishOption {
    use StarcoinFramework::Config;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Signer;

    spec module {
        pragma verify = false; // break after enabling v2 compilation scheme
        pragma aborts_if_is_strict = true;

    }
    spec fun spec_is_script_allowed(addr: address) : bool{
        let publish_option = Config::get_by_address<TransactionPublishOption>(addr);
        publish_option.script_allowed
    }

    spec fun spec_is_module_allowed(addr: address) : bool{
        let publish_option = Config::get_by_address<TransactionPublishOption>(addr);
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
        Timestamp::assert_genesis();
        assert!(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            Errors::requires_address(EPROLOGUE_ACCOUNT_DOES_NOT_EXIST),
        );
        let transaction_publish_option = Self::new_transaction_publish_option(script_allowed, module_publishing_allowed);
        Config::publish_new_config(
            account,
            transaction_publish_option,
        );
    }

    spec initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        include Config::PublishNewConfigAbortsIf<TransactionPublishOption>;
        include Config::PublishNewConfigEnsures<TransactionPublishOption>;
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
        let publish_option = Config::get_by_address<TransactionPublishOption>(account);
        publish_option.script_allowed
    }

    spec is_script_allowed {
        include Config::AbortsIfConfigNotExist<TransactionPublishOption>{
            addr: account
        };
    }

    /// Check if a sender can publish a module
    public fun is_module_allowed(account: address): bool {
        let publish_option = Config::get_by_address<TransactionPublishOption>(account);
        publish_option.module_publishing_allowed
    }

    spec is_module_allowed {
        include Config::AbortsIfConfigNotExist<TransactionPublishOption>{
            addr: account
        };
    }

    spec schema AbortsIfTxnPublishOptionNotExist {
        include Config::AbortsIfConfigNotExist<TransactionPublishOption>{
            addr: CoreAddresses::SPEC_GENESIS_ADDRESS()
        };
    }

    spec schema AbortsIfTxnPublishOptionNotExistWithBool {
        is_script_or_package : bool;
        aborts_if is_script_or_package && !exists<Config::Config<TransactionPublishOption>>(CoreAddresses::GENESIS_ADDRESS());
    }

}
}