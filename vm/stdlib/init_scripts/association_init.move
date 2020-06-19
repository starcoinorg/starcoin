script {
use 0x1::CoreAddresses;
use 0x1::Account;
use 0x1::Block;
use 0x1::Signer;
use 0x1::TransactionTimeout;
use 0x1::Config;
use 0x1::Coin;
use 0x1::Timestamp;
use 0x1::Association;

fun association_init(association: &signer,
    genesis_auth_key: vector<u8>) {

    let dummy_auth_key_prefix = x"00000000000000000000000000000000";

    // Association root setup
    Association::initialize(association);
    Association::grant_privilege<Coin::AddCurrency>(association, association);

    Account::create_genesis_account(
                Signer::address_of(association),
                copy dummy_auth_key_prefix,
    );

    Account::create_genesis_account(
        Config::default_config_address(),
        copy dummy_auth_key_prefix
    );

    Account::create_genesis_account(
        CoreAddresses::MINT_ADDRESS(),
        copy dummy_auth_key_prefix
    );

    Account::create_genesis_account(
        CoreAddresses::TRANSACTION_FEE_ADDRESS(),
        copy dummy_auth_key_prefix
    );

    Account::create_genesis_account(
        CoreAddresses::MINT_ADDRESS(),
        copy dummy_auth_key_prefix
    );

    TransactionTimeout::initialize(association);

    Block::initialize_block_metadata(association);
    Timestamp::initialize(association);

    let assoc_rotate_key_cap = Account::extract_key_rotation_capability(association);
    Account::rotate_authentication_key(&assoc_rotate_key_cap, copy genesis_auth_key);
    Account::restore_key_rotation_capability(assoc_rotate_key_cap);
}
}
