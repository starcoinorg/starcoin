address 0x1 {

module TransactionPublishOption {
    use 0x1::Vector;
    use 0x1::Config;
    use 0x1::Timestamp;
    use 0x1::CoreAddresses;
    use 0x1::ErrorCode;
    use 0x1::Signer;

    const SCRIPT_HASH_LENGTH: u64 = 32;

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
        script_allow_list: vector<vector<u8>>,
        module_publishing_allowed: bool,
    ) {
        assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST());

        Config::publish_new_config(
            account,
            TransactionPublishOption {
                script_allow_list,
                module_publishing_allowed
            }
        );
    }

    // Check if sender can execute script with `hash`
    public fun is_script_allowed(account: &signer, hash: &vector<u8>): bool {
        let publish_option = Config::get<TransactionPublishOption>(account);

        Vector::is_empty(&publish_option.script_allow_list)
            || Vector::contains(&publish_option.script_allow_list, hash)
    }

    // Check if a sender can publish a module
    public fun is_module_allowed(account: &signer): bool {
        let publish_option = Config::get<TransactionPublishOption>(account);

        publish_option.module_publishing_allowed
    }

    // Add `new_hash` to the list of script hashes that is allowed to be executed by the network.
    public fun add_to_script_allow_list(account: &signer, new_hash: vector<u8>) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST());
        assert(Vector::length(&new_hash) == SCRIPT_HASH_LENGTH, ErrorCode::EINVALID_ARGUMENT());

        let publish_option = Config::get<TransactionPublishOption>(account);
        if (Vector::contains(&publish_option.script_allow_list, &new_hash)) {
            abort EALLOWLIST_ALREADY_CONTAINS_SCRIPT
        };
        Vector::push_back(&mut publish_option.script_allow_list, new_hash);

        Config::set<TransactionPublishOption>(account, publish_option);
    }

    // Allow the execution of arbitrary script or not.
    public fun set_open_script(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST());

        let publish_option = Config::get<TransactionPublishOption>(account);

        publish_option.script_allow_list = Vector::empty();
        Config::set<TransactionPublishOption>(account, publish_option);
    }

    // Allow module publishing from arbitrary sender or not.
    public fun set_open_module(account: &signer, open_module: bool) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST());

        let publish_option = Config::get<TransactionPublishOption>(account);

        publish_option.module_publishing_allowed = open_module;
        Config::set<TransactionPublishOption>(account, publish_option);
    }
}
}
