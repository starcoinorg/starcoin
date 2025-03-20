address StarcoinFramework {
/// The module provides a general implmentation of configuration for onchain contracts.    
module Config {
    use StarcoinFramework::Event;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Option::{Self, Option};
    use StarcoinFramework::Errors;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict;
    }

    /// A generic singleton resource that holds a value of a specific type.
    struct Config<ConfigValue: copy + drop + store> has key { payload: ConfigValue }

    /// Accounts with this privilege can modify config of type ConfigValue under account_address
    struct ModifyConfigCapability<ConfigValue: copy + drop + store> has store {
        account_address: address,
        events: Event::EventHandle<ConfigChangeEvent<ConfigValue>>,
    }

    /// A holder for ModifyConfigCapability, for extract and restore ModifyConfigCapability.
    struct ModifyConfigCapabilityHolder<ConfigValue: copy + drop + store> has key, store {
        cap: Option<ModifyConfigCapability<ConfigValue>>,
    }

    /// Event emitted when config value is changed.
    struct ConfigChangeEvent<ConfigValue: copy + drop + store> has drop, store {
        account_address: address,
        value: ConfigValue,
    }

    const ECONFIG_VALUE_DOES_NOT_EXIST: u64 = 13;
    const ECAPABILITY_HOLDER_NOT_EXISTS: u64 = 101;



        spec fun spec_get<ConfigValue>(addr: address): ConfigValue {
            global<Config<ConfigValue>>(addr).payload
        }


    /// Get a copy of `ConfigValue` value stored under `addr`.
    public fun get_by_address<ConfigValue: copy + drop + store>(addr: address): ConfigValue acquires Config {
        assert!(exists<Config<ConfigValue>>(addr), Errors::invalid_state(ECONFIG_VALUE_DOES_NOT_EXIST));
        *&borrow_global<Config<ConfigValue>>(addr).payload
    }

    spec get_by_address {
        aborts_if !exists<Config<ConfigValue>>(addr);
        ensures exists<Config<ConfigValue>>(addr);
    }
    /// Check whether the config of `ConfigValue` type exists under `addr`.
    public fun config_exist_by_address<ConfigValue: copy + drop + store>(addr: address): bool {
        exists<Config<ConfigValue>>(addr)
    }

    spec config_exist_by_address {
        aborts_if false;
    }

    /// Set a config item to a new value with capability stored under signer
    public fun set<ConfigValue: copy + drop + store>(account: &signer, payload: ConfigValue) acquires Config,ModifyConfigCapabilityHolder{
        let signer_address = Signer::address_of(account);
        assert!(exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address), Errors::requires_capability(ECAPABILITY_HOLDER_NOT_EXISTS));
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address);
        assert!(Option::is_some(&cap_holder.cap), Errors::requires_capability(ECAPABILITY_HOLDER_NOT_EXISTS));
        set_with_capability(Option::borrow_mut(&mut cap_holder.cap), payload)
    }

    spec set {
        pragma verify = false;

        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
        aborts_if !Option::is_some<ModifyConfigCapability<ConfigValue>>(spec_cap<ConfigValue>(Signer::address_of(account)));
        aborts_if !exists<Config<ConfigValue>>(Option::borrow<ModifyConfigCapability<ConfigValue>>(spec_cap<ConfigValue>(Signer::address_of(account))).account_address);
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
    }


        spec fun spec_cap<ConfigValue>(addr: address): Option<ModifyConfigCapability<ConfigValue>> {
            global<ModifyConfigCapabilityHolder<ConfigValue>>(addr).cap
        }


    /// Set a config item to a new value with cap.
    public fun set_with_capability<ConfigValue: copy + drop + store>(cap: &mut ModifyConfigCapability<ConfigValue>, payload: ConfigValue) acquires Config{
        let addr = cap.account_address;
        assert!(exists<Config<ConfigValue>>(addr), Errors::invalid_state(ECONFIG_VALUE_DOES_NOT_EXIST));
        let config = borrow_global_mut<Config<ConfigValue>>(addr);
        config.payload = copy payload;
        emit_config_change_event(cap, payload);
    }

    spec set_with_capability {
        aborts_if !exists<Config<ConfigValue>>(cap.account_address);
        ensures exists<Config<ConfigValue>>(cap.account_address);
    }

    /// Publish a new config item. The caller will use the returned ModifyConfigCapability to specify the access control
    /// policy for who can modify the config.
    public fun publish_new_config_with_capability<ConfigValue: copy + drop + store>(
        account: &signer,
        payload: ConfigValue,
    ): ModifyConfigCapability<ConfigValue> acquires ModifyConfigCapabilityHolder{
        publish_new_config<ConfigValue>(account, payload);
        extract_modify_config_capability<ConfigValue>(account)
    }

    spec publish_new_config_with_capability {
        aborts_if exists<Config<ConfigValue>>(Signer::address_of(account));
        aborts_if exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
        ensures exists<Config<ConfigValue>>(Signer::address_of(account));
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
    }

    /// Publish a new config item under account address.
    public fun publish_new_config<ConfigValue: copy + drop + store>(account: &signer, payload: ConfigValue) {
        move_to(account, Config<ConfigValue>{ payload });
        let cap = ModifyConfigCapability<ConfigValue> {account_address: Signer::address_of(account), events: Event::new_event_handle<ConfigChangeEvent<ConfigValue>>(account)};
        move_to(account, ModifyConfigCapabilityHolder{cap: Option::some(cap)});
    }

    spec publish_new_config {
        aborts_if exists<Config<ConfigValue>>(Signer::address_of(account));
        aborts_if exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
        ensures exists<Config<ConfigValue>>(Signer::address_of(account));
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
    }

    spec schema PublishNewConfigAbortsIf<ConfigValue> {
        account: signer;
        aborts_if exists<Config<ConfigValue>>(Signer::address_of(account));
        aborts_if exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
    }

    spec schema AbortsIfConfigNotExist<ConfigValue> {
        addr: address;

        aborts_if !exists<Config<ConfigValue>>(addr);
    }

    spec schema AbortsIfConfigOrCapabilityNotExist<ConfigValue> {
        addr: address;

        aborts_if !exists<Config<ConfigValue>>(addr);
        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(addr);
    }

    spec schema PublishNewConfigEnsures<ConfigValue> {
        account: signer;
        ensures exists<Config<ConfigValue>>(Signer::address_of(account));
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
    }

    /// Extract account's ModifyConfigCapability for ConfigValue type
    public fun extract_modify_config_capability<ConfigValue: copy + drop + store>(account: &signer): ModifyConfigCapability<ConfigValue> acquires ModifyConfigCapabilityHolder{
        let signer_address = Signer::address_of(account);
        assert!(exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address), Errors::requires_capability(ECAPABILITY_HOLDER_NOT_EXISTS));
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(signer_address);
        Option::extract(&mut cap_holder.cap)
    }
    spec schema AbortsIfCapNotExist<ConfigValue> {
        account: address;
        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(account);
        aborts_if Option::is_none<ModifyConfigCapability<ConfigValue>>(global<ModifyConfigCapabilityHolder<ConfigValue>>(account).cap);
    }

    spec extract_modify_config_capability {
        include AbortsIfCapNotExist<ConfigValue>{account: Signer::address_of(account)};
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account));
        ensures Option::is_none<ModifyConfigCapability<ConfigValue>>(global<ModifyConfigCapabilityHolder<ConfigValue>>(Signer::address_of(account)).cap);
    }

    /// Restore account's ModifyConfigCapability
    public fun restore_modify_config_capability<ConfigValue: copy + drop + store>(cap: ModifyConfigCapability<ConfigValue>) acquires ModifyConfigCapabilityHolder{
        let cap_holder = borrow_global_mut<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        Option::fill(&mut cap_holder.cap, cap);
    }

    spec restore_modify_config_capability {
        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        aborts_if Option::is_some<ModifyConfigCapability<ConfigValue>>(global<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address).cap);
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        ensures Option::is_some<ModifyConfigCapability<ConfigValue>>(global<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address).cap);
    }

    /// Destroy the given ModifyConfigCapability
    public fun destroy_modify_config_capability<ConfigValue: copy + drop + store>(cap: ModifyConfigCapability<ConfigValue>) {
        let ModifyConfigCapability{account_address:_, events} = cap;
        Event::destroy_handle(events)
    }

    spec destroy_modify_config_capability {
        aborts_if false;
    }

    /// Return the address of the given ModifyConfigCapability
    public fun account_address<ConfigValue: copy + drop + store>(cap: &ModifyConfigCapability<ConfigValue>): address {
        cap.account_address
    }
    spec account_address {
        aborts_if false;
    }

    /// Emit a config change event.
    fun emit_config_change_event<ConfigValue: copy + drop + store>(cap: &mut ModifyConfigCapability<ConfigValue>, value: ConfigValue) {
        Event::emit_event<ConfigChangeEvent<ConfigValue>>(
            &mut cap.events,
            ConfigChangeEvent {
                account_address: cap.account_address,
                value: value,
            },
        );
    }

    spec emit_config_change_event {
        aborts_if false;
    }
}
}
