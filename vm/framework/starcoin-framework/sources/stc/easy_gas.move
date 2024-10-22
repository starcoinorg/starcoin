module starcoin_framework::easy_gas {

    use std::bcs;
    use std::error;
    use std::signer;

    use starcoin_framework::account;
    use starcoin_framework::coin;
    use starcoin_framework::oracle_price;
    use starcoin_framework::reserved_accounts_signer;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::system_addresses;
    use starcoin_std::type_info;

    const EBAD_TRANSACTION_FEE_TOKEN: u64 = 18;

    struct STCToken<phantom TokenType: store> has copy, store, drop {}

    struct GasTokenEntry has key, store, drop {
        account_address: address,
        module_name: vector<u8>,
        struct_name: vector<u8>,
        data_source: address,
    }

    struct GasFeeAddress has key, store {
        gas_fee_address: address,
        cap: account::SignerCapability,
    }

    public fun initialize(
        sender: &signer,
        token_account_address: address,
        token_module_name: vector<u8>,
        token_struct_name: vector<u8>,
        data_source: address,
    ) acquires GasTokenEntry {
        register_gas_token(sender, token_account_address, token_module_name, token_struct_name, data_source);
        create_gas_fee_address(sender);
    }

    public fun register_oracle<TokenType: store>(sender: &signer, precision: u8) {
        oracle_price::register_oracle<STCToken<TokenType>>(sender, precision);
        // let genesis_account =
        //     reserved_accounts_signer::get_stored_signer(system_addresses::get_starcoin_framework());
        // // todo:check gas token entry
        // coin::register<TokenType>(&genesis_account);
    }

    public fun init_oracle_source<TokenType: store>(sender: &signer, init_value: u128) {
        oracle_price::init_data_source<STCToken<TokenType>>(sender, init_value);
    }

    public fun update_oracle<TokenType: store>(sender: &signer, value: u128) {
        oracle_price::update<STCToken<TokenType>>(sender, value);
    }

    public fun get_scaling_factor<TokenType: store>(): u128 {
        oracle_price::get_scaling_factor<STCToken<TokenType>>()
    }

    public fun gas_oracle_read<TokenType: store>(): u128 acquires GasTokenEntry {
        let data_source = get_data_source_address<TokenType>();
        oracle_price::read<STCToken<TokenType>>(data_source)
    }


    fun register_gas_token(
        sender: &signer,
        account_address: address,
        module_name: vector<u8>,
        struct_name: vector<u8>,
        data_source: address,
    ) acquires GasTokenEntry {
        system_addresses::assert_starcoin_framework(sender);

        let genesis_account =
            reserved_accounts_signer::get_stored_signer(system_addresses::get_starcoin_framework());
        let gas_token_entry = GasTokenEntry { account_address, module_name, struct_name, data_source };
        if (exists<GasTokenEntry>(signer::address_of(&genesis_account))) {
            move_from<GasTokenEntry>(signer::address_of(&genesis_account));
        };
        move_to(&genesis_account, gas_token_entry);
    }

    fun get_data_source_address<TokenType: store>(): address acquires GasTokenEntry {
        let token_type_info = type_info::type_of<TokenType>();
        let genesis = system_addresses::get_starcoin_framework();
        let gas_token_entry = borrow_global<GasTokenEntry>(genesis);
        assert!(type_info::module_name(&token_type_info) == *&gas_token_entry.module_name
            && type_info::account_address(&token_type_info) == *&gas_token_entry.account_address
            && type_info::struct_name(&token_type_info) == *&gas_token_entry.struct_name,
            error::invalid_argument(EBAD_TRANSACTION_FEE_TOKEN)
        );
        gas_token_entry.data_source
    }

    fun create_gas_fee_address(sender: &signer) {
        system_addresses::assert_starcoin_framework(sender);
        let genesis_account =
            reserved_accounts_signer::get_stored_signer(system_addresses::get_starcoin_framework());
        let (gas_fee_signer, cap) = account::create_resource_account(
            &genesis_account,
            bcs::to_bytes(&signer::address_of(sender))
        );
        coin::register<STC>(&gas_fee_signer);
        //let gas_fee_signer = account::create_signer_with_cap(&cap);
        // account::set_auto_accept_token(&gas_fee_signer, true);
        move_to(&genesis_account, GasFeeAddress {
            gas_fee_address: signer::address_of(&gas_fee_signer),
            cap
        });
    }

    public fun get_gas_fee_address(): address acquires GasFeeAddress {
        borrow_global<GasFeeAddress>(system_addresses::get_starcoin_framework()).gas_fee_address
    }

    public fun withdraw_gas_fee<TokenType: store>(_sender: &signer, amount: u128) acquires GasFeeAddress {
        let gas_fee_address_entry =
            borrow_global<GasFeeAddress>(system_addresses::get_starcoin_framework());
        let gas_fee_signer = account::create_signer_with_capability(&gas_fee_address_entry.cap);
        // let withdraw_cap = extract_withdraw_capability(&gas_fee_signer);
        // let token = withdraw_with_capability<TokenType>(&withdraw_cap, amount);
        // restore_withdraw_capability(withdraw_cap);
        // deposit(CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), token);

        coin::deposit(
            system_addresses::get_core_resource_address(),
            coin::withdraw<TokenType>(&gas_fee_signer, (amount as u64))
        );
    }
}
