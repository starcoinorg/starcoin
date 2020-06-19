// The genesis module. This defines the majority of the Move functions that
// are executed, and the order in which they are executed in genesis. Note
// however, that there are certain calls that remain in Rust code in
// genesis (for now).
address 0x1 {
module Genesis {
    use 0x1::Association;
    use 0x1::STC::{Self, STC};
    use 0x1::Coin;
    use 0x1::Account;
    use 0x1::Block;
    use 0x1::Config;
    use 0x1::TransactionTimeout;
    use 0x1::Timestamp;
    use 0x1::Version;
    use 0x1::Signer;

    
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

        // Create the config account
        Account::create_genesis_account(
            Config::default_config_address(),
            copy dummy_auth_key_prefix
        );

        Config::initialize(config_account);

        // Currency setup
        Coin::initialize(config_account);

        Account::create_genesis_account(
            Signer::address_of(association),
            copy dummy_auth_key_prefix,
        );

        //Account::initialize(association);
        STC::initialize(association);

        Account::add_currency<STC>(association);
        Account::add_currency<STC>(config_account);

        Account::create_account<STC>(
            Signer::address_of(mint_account),
            copy dummy_auth_key_prefix
        );

        Account::create_account<STC>(
            Signer::address_of(fee_account),
            dummy_auth_key_prefix
        );

        TransactionTimeout::initialize(association);
        Version::initialize(config_account);

        Block::initialize_block_metadata(association);
        Timestamp::initialize(association);

        let assoc_rotate_key_cap = Account::extract_key_rotation_capability(association);
        Account::rotate_authentication_key(&assoc_rotate_key_cap, copy genesis_auth_key);
        Account::restore_key_rotation_capability(assoc_rotate_key_cap);
    }

}
}
