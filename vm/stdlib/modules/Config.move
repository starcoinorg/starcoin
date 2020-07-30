address 0x1 {
module Config {
    use 0x1::Event;
    use 0x1::Signer;
    use 0x1::Option::{Self, Option};
    use 0x1::ErrorCode;

    spec module {
        pragma verify = false;
    }

    // A generic singleton resource that holds a value of a specific type.
    resource struct Config<ConfigValue: copyable> { payload: ConfigValue }

    // Accounts with this privilege can modify config of type ConfigValue under account_address
    resource struct ModifyConfigCapability<ConfigValue: copyable> {
        account_address: address,
        events: Event::EventHandle<ConfigChangeEvent<ConfigValue>>,
    }

    // A holder for ModifyConfigCapability, for extract and restore ModifyConfigCapability.
    resource struct ModifyConfigCapabilityHolder<ConfigValue: copyable> {
        cap: Option<ModifyConfigCapability<ConfigValue>>,
    }

    struct ConfigChangeEvent<ConfigValue: copyable>{
        account_address: address,
        value: ConfigValue,
    }

    // Get a copy of `ConfigValue` value stored under account.
    public fun get<ConfigValue: copyable>(account: &signer): ConfigValue acquires Config {
        let addr = Signer::address_of(account);
        assert(exists<Config<ConfigValue>>(addr), ErrorCode::ECONFIG_VALUE_DOES_NOT_EXIST());
        *&borrow_global<Config<ConfigValue>>(addr).payload
    }

    // Get a copy of `ConfigValue` value stored under `addr`.
    public fun get_by_address<ConfigValue: copyable>(addr: address): ConfigValue acquires Config {
        assert(exists<Config<ConfigValue>>(addr), ErrorCode::ECONFIG_VALUE_DOES_NOT_EXIST());
        *&borrow_global<Config<ConfigValue>>(addr).payload
    }

    // Set a config item to a new value with capability stored under signer
    public fun set<ConfigValue: copyable>(account: &signer, payload: ConfigValue) acquires Config,ModifyConfigCapabilityHolder{
        let signer_address = Signer::address_of(account);
        //TODO define no capability error code.
        assert(exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address), 24);
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address);
        assert(Option::is_some(&cap_holder.cap), 24);
        set_with_capability(Option::borrow_mut(&mut cap_holder.cap), payload)
    }

    // Set a config item to a new value with cap.
    public fun set_with_capability<ConfigValue: copyable>(cap: &mut ModifyConfigCapability<ConfigValue>, payload: ConfigValue) acquires Config{
        let addr = cap.account_address;
        assert(exists<Config<ConfigValue>>(addr), ErrorCode::ECONFIG_VALUE_DOES_NOT_EXIST());
        let config = borrow_global_mut<Config<ConfigValue>>(addr);
        config.payload = copy payload;
        emit_config_change_event(cap, payload);
    }

    // Publish a new config item. The caller will use the returned ModifyConfigCapability to specify the access control
    // policy for who can modify the config.
    public fun publish_new_config_with_capability<ConfigValue: copyable>(
        account: &signer,
        payload: ConfigValue,
    ): ModifyConfigCapability<ConfigValue> acquires ModifyConfigCapabilityHolder{
        publish_new_config<ConfigValue>(account, payload);
        extract_modify_config_capability<ConfigValue>(account)
    }

    // Publish a new config item under account address.
    public fun publish_new_config<ConfigValue: copyable>(account: &signer, payload: ConfigValue) {
        move_to(account, Config{ payload });
        let cap = ModifyConfigCapability<ConfigValue> {account_address: Signer::address_of(account), events: Event::new_event_handle<ConfigChangeEvent<ConfigValue>>(account)};
        move_to(account, ModifyConfigCapabilityHolder{cap: Option::some(cap)});
    }

    // Extract account's ModifyConfigCapability for ConfigValue type
    public fun extract_modify_config_capability<ConfigValue: copyable>(account: &signer): ModifyConfigCapability<ConfigValue> acquires ModifyConfigCapabilityHolder{
        let signer_address = Signer::address_of(account);
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address);
        Option::extract(&mut cap_holder.cap)
    }

    // Restore account's ModifyConfigCapability
    public fun restore_modify_config_capability<ConfigValue: copyable>(cap: ModifyConfigCapability<ConfigValue>) acquires ModifyConfigCapabilityHolder{
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        Option::fill(&mut cap_holder.cap, cap);
    }

    public fun destory_modify_config_capability<ConfigValue: copyable>(cap: ModifyConfigCapability<ConfigValue>) {
        let ModifyConfigCapability{account_address:_, events} = cap;
        Event::destroy_handle(events)
    }

    // Emit a config change event.
    fun emit_config_change_event<ConfigValue: copyable>(cap: &mut ModifyConfigCapability<ConfigValue>, value: ConfigValue) {
        Event::emit_event<ConfigChangeEvent<ConfigValue>>(
            &mut cap.events,
            ConfigChangeEvent {
                account_address: cap.account_address,
                value: value,
            },
        );
    }

}
}
