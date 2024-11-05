/// The module provides a general implmentation of configuration for onchain contracts.
module starcoin_framework::on_chain_config {

    use std::error;
    use std::option;
    use std::signer;
    use starcoin_std::debug;

    use starcoin_framework::account;
    use starcoin_framework::event;

    /// A generic singleton resource that holds a value of a specific type.
    struct Config<ConfigValue: copy + drop + store> has key { payload: ConfigValue }

    /// Accounts with this privilege can modify config of type ConfigValue under account_address
    struct ModifyConfigCapability<ConfigValue: copy + drop + store> has store {
        account_address: address,
        events: event::EventHandle<ConfigChangeEvent<ConfigValue>>,
    }

    /// A holder for ModifyConfigCapability, for extraction and restoration of ModifyConfigCapability.
    struct ModifyConfigCapabilityHolder<ConfigValue: copy + drop + store> has key, store {
        cap: option::Option<ModifyConfigCapability<ConfigValue>>,
    }

    /// Event emitted when config value is changed.
    struct ConfigChangeEvent<ConfigValue: copy + drop + store> has drop, store {
        account_address: address,
        value: ConfigValue,
    }

    const ECONFIG_VALUE_DOES_NOT_EXIST: u64 = 13;
    const ECAPABILITY_HOLDER_NOT_EXISTS: u64 = 101;

    /// Get a copy of `ConfigValue` value stored under `addr`.
    public fun get_by_address<ConfigValue: copy + drop + store>(addr: address): ConfigValue acquires Config {
        assert!(exists<Config<ConfigValue>>(addr), error::invalid_state(ECONFIG_VALUE_DOES_NOT_EXIST));
        *&borrow_global<Config<ConfigValue>>(addr).payload
    }

    /// Check whether the config of `ConfigValue` type exists under `addr`.
    public fun config_exist_by_address<ConfigValue: copy + drop + store>(addr: address): bool {
        exists<Config<ConfigValue>>(addr)
    }


    /// Set a config item to a new value with capability stored under signer
    public fun set<ConfigValue: copy + drop + store>(
        account: &signer,
        payload: ConfigValue,
    ) acquires Config, ModifyConfigCapabilityHolder {
        let signer_address = signer::address_of(account);
        debug::print(&std::string::utf8(b"on_chain_config::get "));
        debug::print(&payload);
        assert!(
            exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address),
            error::resource_exhausted(ECAPABILITY_HOLDER_NOT_EXISTS),
        );
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address);
        assert!(option::is_some(&cap_holder.cap), error::resource_exhausted(ECAPABILITY_HOLDER_NOT_EXISTS));
        set_with_capability(option::borrow_mut(&mut cap_holder.cap), payload);
    }


    /// Set a config item to a new value with cap.
    public fun set_with_capability<ConfigValue: copy + drop + store>(
        cap: &mut ModifyConfigCapability<ConfigValue>,
        payload: ConfigValue,
    ) acquires Config {
        let addr = cap.account_address;
        assert!(exists<Config<ConfigValue>>(addr), error::invalid_state(ECONFIG_VALUE_DOES_NOT_EXIST));
        let config = borrow_global_mut<Config<ConfigValue>>(addr);
        config.payload = copy payload;
        emit_config_change_event(cap, payload);
    }

    /// Publish a new config item. The caller will use the returned ModifyConfigCapability to specify the access control
    /// policy for who can modify the config.
    public fun publish_new_config_with_capability<ConfigValue: copy + drop + store>(
        account: &signer,
        payload: ConfigValue,
    ): ModifyConfigCapability<ConfigValue> acquires ModifyConfigCapabilityHolder {
        publish_new_config<ConfigValue>(account, payload);
        extract_modify_config_capability<ConfigValue>(account)
    }


    /// Publish a new config item under account address.
    public fun publish_new_config<ConfigValue: copy + drop + store>(account: &signer, payload: ConfigValue) {
        move_to(account, Config<ConfigValue> { payload });
        let cap = ModifyConfigCapability<ConfigValue> {
            account_address: signer::address_of(account),
            events: account::new_event_handle<ConfigChangeEvent<ConfigValue>>(account),
        };
        move_to<ModifyConfigCapabilityHolder<ConfigValue>>(
            account,
            ModifyConfigCapabilityHolder { cap: option::some(cap) }
        );
    }

    /// Extract account's ModifyConfigCapability for ConfigValue type
    public fun extract_modify_config_capability<ConfigValue: copy + drop + store>(
        account: &signer,
    ): ModifyConfigCapability<ConfigValue> acquires ModifyConfigCapabilityHolder {

        debug::print(&std::string::utf8(b"on_chain_config::extract_modify_config_capability "));
        debug::print_stack_trace();

        let signer_address = signer::address_of(account);
        assert!(
            exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address),
            error::permission_denied(ECAPABILITY_HOLDER_NOT_EXISTS)
        );
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address);
        option::extract(&mut cap_holder.cap)
    }


    /// Restore account's ModifyConfigCapability
    public fun restore_modify_config_capability<ConfigValue: copy + drop + store>(
        cap: ModifyConfigCapability<ConfigValue>,
    ) acquires ModifyConfigCapabilityHolder {
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        option::fill(&mut cap_holder.cap, cap);
    }

    /// Destroy the given ModifyConfigCapability
    public fun destroy_modify_config_capability<ConfigValue: copy + drop + store>(
        cap: ModifyConfigCapability<ConfigValue>,
    ) {
        let ModifyConfigCapability { account_address: _, events } = cap;
        event::destroy_handle(events)
    }


    /// Return the address of the given ModifyConfigCapability
    public fun account_address<ConfigValue: copy + drop + store>(cap: &ModifyConfigCapability<ConfigValue>): address {
        cap.account_address
    }

    /// Emit a config change event.
    fun emit_config_change_event<ConfigValue: copy + drop + store>(
        cap: &mut ModifyConfigCapability<ConfigValue>,
        value: ConfigValue,
    ) {
        event::emit_event<ConfigChangeEvent<ConfigValue>>(
            &mut cap.events,
            ConfigChangeEvent {
                account_address: cap.account_address,
                value,
            },
        );
    }
}
