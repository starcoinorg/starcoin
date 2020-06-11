// The genesis module. This defines the majority of the Move functions that
// are executed, and the order in which they are executed in genesis. Note
// however, that there are certain calls that remain in Rust code in
// genesis (for now).
address 0x0 {
module Genesis {
    use 0x0::Association;
    use 0x0::Event;
    use 0x0::STC;
    use 0x0::Coin;
    use 0x0::Account;
    use 0x0::Block;
    use 0x0::Config;
    use 0x0::TransactionTimeout;
    use 0x0::Testnet;
    use 0x0::Timestamp;
    use 0x0::Version;
    use 0x0::Signer;

    
    fun initialize(
            association: &signer,
            config_account: &signer,
            fee_account: &signer,
            mint_account: &signer,
            genesis_auth_key: vector<u8>
     ) {
        let dummy_auth_key_prefix = x"00000000000000000000000000000000";

        // Association root setup
        Association::initialize(association);
        Association::grant_privilege<Coin::AddCurrency>(association, association);

        // On-chain config setup
        Event::publish_generator(config_account);
        Config::initialize(config_account, association);

        // Currency setup
        Coin::initialize(config_account);

        // Set that this is testnet
        Testnet::initialize(association);

        // Event and currency setup
        Event::publish_generator(association);

        //Account::initialize(association);
        STC::initialize(association);

        Account::create_genesis_account<STC::T>(
            Signer::address_of(association),
            copy dummy_auth_key_prefix,
        );


        // Create the config account
        Account::create_genesis_account<STC::T>(
            Config::default_config_address(),
            copy dummy_auth_key_prefix
        );

        Account::create_account<STC::T>(
            Signer::address_of(mint_account),
            copy dummy_auth_key_prefix
        );

        Account::create_account<STC::T>(
            Signer::address_of(fee_account),
            dummy_auth_key_prefix
        );

        TransactionTimeout::initialize(association);
        Version::initialize(config_account);

        Block::initialize_block_metadata(association);
        Timestamp::initialize(association);
        Account::rotate_authentication_key(association, copy genesis_auth_key);
    }

}
}
