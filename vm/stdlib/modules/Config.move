address 0x1 {
module Config {
    use 0x1::Event;
    use 0x1::Signer;
    use 0x1::Option::{Self, Option};
    use 0x1::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    // A generic singleton resource that holds a value of a specific type.
    resource struct Config<ConfigValue: copyable> { payload: ConfigValue }

    // Accounts with this privilege can modify config of type ConfigValue under account_address
    resource struct ModifyConfigCapability<ConfigValue: copyable> {
        account_address: address,
        /// FIXME: events should put into Config resource.
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

    const ECONFIG_VALUE_DOES_NOT_EXIST: u64 = 13; // do not change
    const ECAPABILITY_HOLDER_NOT_EXISTS: u64 = 101;


    spec module {
        define spec_get<ConfigValue>(addr: address): ConfigValue {
            global<Config<ConfigValue>>(addr).payload
        }
    }

    // Get a copy of `ConfigValue` value stored under `addr`.
    public fun get_by_address<ConfigValue: copyable>(addr: address): ConfigValue acquires Config {
        assert(exists<Config<ConfigValue>>(addr), Errors::invalid_state(ECONFIG_VALUE_DOES_NOT_EXIST));
        *&borrow_global<Config<ConfigValue>>(addr).payload
    }

    spec fun get_by_address {
        aborts_if !exists<Config<ConfigValue>>(addr);
        ensures exists<Config<ConfigValue>>(addr);
    }

    // Set a config item to a new value with capability stored under signer
    public fun set<ConfigValue: copyable>(account: &signer, payload: ConfigValue) acquires Config,ModifyConfigCapabilityHolder{
        let signer_address = Signer::address_of(account);
        //TODO define no capability error code.
        assert(exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address), Errors::requires_capability(ECAPABILITY_HOLDER_NOT_EXISTS));
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address);
        assert(Option::is_some(&cap_holder.cap), Errors::requires_capability(ECAPABILITY_HOLDER_NOT_EXISTS));
        set_with_capability(Option::borrow_mut(&mut cap_holder.cap), payload)
    }

    spec fun set {
        pragma verify = false;

        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
        aborts_if !Option::spec_is_some<ModifyConfigCapability<ConfigValue>>(spec_cap<ConfigValue>(Signer::spec_address_of(account)));
        //Todo: why below aborts_if does not work?
        aborts_if !exists<Config<ConfigValue>>(Option::spec_get<ModifyConfigCapability<ConfigValue>>(spec_cap<ConfigValue>(Signer::spec_address_of(account))).account_address);
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
    }

    spec module {
        define spec_cap<ConfigValue>(addr: address): Option<ModifyConfigCapability<ConfigValue>> {
            global<ModifyConfigCapabilityHolder<ConfigValue>>(addr).cap
        }
    }

    // Set a config item to a new value with cap.
    public fun set_with_capability<ConfigValue: copyable>(cap: &mut ModifyConfigCapability<ConfigValue>, payload: ConfigValue) acquires Config{
        let addr = cap.account_address;
        assert(exists<Config<ConfigValue>>(addr), Errors::invalid_state(ECONFIG_VALUE_DOES_NOT_EXIST));
        let config = borrow_global_mut<Config<ConfigValue>>(addr);
        config.payload = copy payload;
        emit_config_change_event(cap, payload);
    }

    spec fun set_with_capability {
        aborts_if !exists<Config<ConfigValue>>(cap.account_address);
        ensures exists<Config<ConfigValue>>(cap.account_address);
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

    spec fun publish_new_config_with_capability {
        aborts_if exists<Config>(Signer::spec_address_of(account));
        aborts_if exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
        ensures exists<Config>(Signer::spec_address_of(account));
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
    }

    // Publish a new config item under account address.
    public fun publish_new_config<ConfigValue: copyable>(account: &signer, payload: ConfigValue) {
        move_to(account, Config<ConfigValue>{ payload });
        let cap = ModifyConfigCapability<ConfigValue> {account_address: Signer::address_of(account), events: Event::new_event_handle<ConfigChangeEvent<ConfigValue>>(account)};
        move_to(account, ModifyConfigCapabilityHolder{cap: Option::some(cap)});
    }

    spec fun publish_new_config {
        aborts_if exists<Config<ConfigValue>>(Signer::spec_address_of(account));
        aborts_if exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
        ensures exists<Config<ConfigValue>>(Signer::spec_address_of(account));
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
    }

    spec schema PublishNewConfigAbortsIf<ConfigValue> {
        account: signer;
        aborts_if exists<Config<ConfigValue>>(Signer::spec_address_of(account));
        aborts_if exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
    }

    spec schema PublishNewConfigEnsures<ConfigValue> {
        account: signer;
        ensures exists<Config<ConfigValue>>(Signer::spec_address_of(account));
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
    }

    // Extract account's ModifyConfigCapability for ConfigValue type
    public fun extract_modify_config_capability<ConfigValue: copyable>(account: &signer): ModifyConfigCapability<ConfigValue> acquires ModifyConfigCapabilityHolder{
        let signer_address = Signer::address_of(account);
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address);
        Option::extract(&mut cap_holder.cap)
    }

    spec fun extract_modify_config_capability {
        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
        aborts_if Option::spec_is_none<ModifyConfigCapability<ConfigValue>>(global<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account)).cap);
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account));
        ensures Option::spec_is_none<ModifyConfigCapability<ConfigValue>>(global<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::spec_address_of(account)).cap);
    }

    // Restore account's ModifyConfigCapability
    public fun restore_modify_config_capability<ConfigValue: copyable>(cap: ModifyConfigCapability<ConfigValue>) acquires ModifyConfigCapabilityHolder{
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        Option::fill(&mut cap_holder.cap, cap);
    }

    spec fun restore_modify_config_capability {
        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        aborts_if Option::spec_is_some<ModifyConfigCapability<ConfigValue>>(global<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address).cap);
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        ensures Option::spec_is_some<ModifyConfigCapability<ConfigValue>>(global<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address).cap);
    }

    public fun destory_modify_config_capability<ConfigValue: copyable>(cap: ModifyConfigCapability<ConfigValue>) {
        let ModifyConfigCapability{account_address:_, events} = cap;
        Event::destroy_handle(events)
    }

    spec fun destory_modify_config_capability {
        aborts_if false;
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

    spec fun emit_config_change_event {
        aborts_if false;
    }

}
}
