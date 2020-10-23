address 0x1 {
module TransactionPublishOption {
    use 0x1::Vector;
    use 0x1::Config;
    use 0x1::Timestamp;
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::Signer;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;

        define spec_is_script_allowed(addr: address, hash: vector<u8>) : bool{
            let publish_option = Config::get_by_address<TransactionPublishOption>(addr);
            len(publish_option.script_allow_list) == 0 ||
                Vector::spec_contains(publish_option.script_allow_list, hash)
        }

        define spec_is_module_allowed(addr: address) : bool{
            let publish_option = Config::get_by_address<TransactionPublishOption>(addr);
            publish_option.module_publishing_allowed
        }
    }

    const SCRIPT_HASH_LENGTH: u64 = 32;

    const EPROLOGUE_ACCOUNT_DOES_NOT_EXIST: u64 = 0;
    const EINVALID_ARGUMENT: u64 = 18;
    /// The script hash has an invalid length
    const EINVALID_SCRIPT_HASH: u64 = 1001;
    /// The script hash already exists in the allowlist
    const EALLOWLIST_ALREADY_CONTAINS_SCRIPT: u64 = 1002;

    /// Defines and holds the publishing policies for the VM. There are three possible configurations:
    /// 1. No module publishing, only allowlisted scripts are allowed.
    /// 2. No module publishing, custom scripts are allowed.
    /// 3. Both module publishing and custom scripts are allowed.
    /// We represent these as the following resource.
    struct TransactionPublishOption {
        // Only script hashes in the following list can be executed. If the vector is empty, no
        // limitation would be enforced.
        script_allow_list: vector<vector<u8>>,
        // Anyone can publish new module if this flag is set to true.
        module_publishing_allowed: bool,
    }

    public fun initialize(
        account: &signer,
        merged_script_allow_list: vector<u8>,
        module_publishing_allowed: bool,
    ) {
        Timestamp::assert_genesis();
        assert(
            Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(),
            Errors::requires_address(EPROLOGUE_ACCOUNT_DOES_NOT_EXIST),
        );
        let transaction_publish_option = Self::new_transaction_publish_option(merged_script_allow_list, module_publishing_allowed);
        Config::publish_new_config(
            account,
            transaction_publish_option,
        );
    }

    spec fun initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        include Config::PublishNewConfigAbortsIf<TransactionPublishOption>;
        include Config::PublishNewConfigEnsures<TransactionPublishOption>;
    }

    public fun new_transaction_publish_option(
        script_allow_list: vector<u8>,
        module_publishing_allowed: bool,
    ): TransactionPublishOption {
        let list = Vector::empty<vector<u8>>();
        let len = Vector::length(&script_allow_list) / SCRIPT_HASH_LENGTH;
        let i = 0;
        while (i < len){
            let script_hash = Vector::empty<u8>();
            let j = 0;
            while (j < SCRIPT_HASH_LENGTH){
                let index = SCRIPT_HASH_LENGTH * i + j;
                Vector::push_back(
                    &mut script_hash,
                    *Vector::borrow(&script_allow_list, index),
                );
                j = j + 1;
            };
            Vector::push_back<vector<u8>>(&mut list, script_hash);
            i = i + 1;
        };
        TransactionPublishOption { script_allow_list: list, module_publishing_allowed }
    }

    spec fun new_transaction_publish_option {
        aborts_if false;
    }

    // Check if sender can execute script with `hash`
    public fun is_script_allowed(account: address, hash: &vector<u8>): bool {
        let publish_option = Config::get_by_address<TransactionPublishOption>(account);
        Vector::is_empty(&publish_option.script_allow_list) ||
            Vector::contains(&publish_option.script_allow_list, hash)
    }

    spec fun is_script_allowed {
        include Config::AbortsIfConfigNotExist<TransactionPublishOption>{
            addr: account
        };
    }

    // Check if a sender can publish a module
    public fun is_module_allowed(account: address): bool {
        let publish_option = Config::get_by_address<TransactionPublishOption>(account);
        publish_option.module_publishing_allowed
    }

    spec fun is_module_allowed {
        include Config::AbortsIfConfigNotExist<TransactionPublishOption>{
            addr: account
        };
    }

    spec schema AbortsIfTxnPublishOptionNotExist {
        include Config::AbortsIfConfigNotExist<TransactionPublishOption>{
            addr: CoreAddresses::SPEC_GENESIS_ADDRESS()
        };
    }
}
}