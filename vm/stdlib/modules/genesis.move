// The genesis module. This defines the majority of the Move functions that
// are executed, and the order in which they are executed in genesis. Note
// however, that there are certain calls that remain in Rust code in
// genesis (for now).
address 0x0 {
module Genesis {
    use 0x0::AccountTrack;
    use 0x0::AccountType;
    use 0x0::Association;
    use 0x0::Coin1;
    use 0x0::Coin2;
    use 0x0::Empty;
    use 0x0::Event;
    use 0x0::LBR;
    use 0x0::STC;
    use 0x0::Coin;
    use 0x0::Account;
    use 0x0::Block;
    use 0x0::Config;
    use 0x0::TransactionTimeout;
    use 0x0::WriteSetManager;
    use 0x0::TransactionFee;
    use 0x0::Unhosted;
    use 0x0::VASP;
    use 0x0::Testnet;

    fun initialize_association(association_root_addr: address) {
        // Association/cap setup
        Association::initialize();
        Association::apply_for_privilege<Coin::AddCurrency>();
        Association::grant_privilege<Coin::AddCurrency>(association_root_addr);
    }

    fun initialize_accounts(
        association_root_addr: address,
        burn_addr: address,
        mint_addr: address,
        genesis_auth_key: vector<u8>,
    ) {
        let dummy_auth_key = x"00000000000000000000000000000000";

        // Set that this is testnet
        Testnet::initialize();

        // Event and currency setup
        Event::grant_event_generator();
        Coin1::initialize();
        Coin2::initialize();
        LBR::initialize();
        STC::initialize();
        Config::apply_for_creator_privilege();
        Config::grant_creator_privilege(0xA550C18);

        //// Account type setup
        AccountType::register<Unhosted::T>();
        AccountType::register<Empty::T>();
        VASP::initialize();

        AccountTrack::initialize();
        Account::initialize();
        Unhosted::publish_global_limits_definition();
        Account::create_account<STC::T>(
            association_root_addr,
            copy dummy_auth_key,
        );

        // Create the burn account
        Account::create_account<STC::T>(burn_addr, copy dummy_auth_key);

        // Register transaction fee accounts
        // TODO: Need to convert this to a different account type than unhosted.
        Account::create_testnet_account<STC::T>(0xFEE, copy dummy_auth_key);

        // Create the config account
        Account::create_account<STC::T>(Config::default_account_config::config_address(), copy dummy_auth_key);

        // Create the mint account
        Account::create_account<STC::T>(mint_addr, copy dummy_auth_key);

        TransactionTimeout::initialize();
        Block::initialize_block_metadata();
        WriteSetManager::initialize();
        Account::rotate_authentication_key(genesis_auth_key);
    }

    fun initalize_burn_account() {
        Association::apply_for_association();
    }

    fun grant_burn_account(burn_addr: address) {
        Association::grant_association_address(burn_addr)
    }

    fun grant_burn_capabilities_for_sender(auth_key: vector<u8>) {
        Coin::grant_burn_capability_for_sender<Coin1::T>();
        Coin::grant_burn_capability_for_sender<Coin2::T>();
        Coin::grant_burn_capability_for_sender<STC::T>();
        Account::rotate_authentication_key(auth_key);
    }

    fun initialize_txn_fee_account(auth_key: vector<u8>) {
        Account::add_currency<Coin1::T>();
        Account::add_currency<Coin2::T>();
        TransactionFee::initialize_transaction_fees();
        Account::rotate_authentication_key(auth_key);
    }

    fun initialize_config() {
        Event::grant_event_generator();
        Config::initialize_configuration();
        Config::apply_for_creator_privilege();
    }
}
}
