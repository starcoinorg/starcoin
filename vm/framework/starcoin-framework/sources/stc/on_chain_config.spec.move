/// The module provides a general implmentation of configuration for onchain contracts.
spec starcoin_framework::on_chain_config {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    spec fun spec_get<ConfigValue>(addr: address): ConfigValue {
        global<Config<ConfigValue>>(addr).payload
    }

    spec get_by_address {
        aborts_if !exists<Config<ConfigValue>>(addr);
        ensures exists<Config<ConfigValue>>(addr);
        ensures result == spec_get<ConfigValue>(addr);
    }

    spec config_exist_by_address {
        aborts_if false;
        ensures result == exists<Config<ConfigValue>>(addr);
    }

    spec set<ConfigValue: copy + drop + store>(
        account: &signer,
        payload: ConfigValue,
    ) {
        use std::option;
        use starcoin_framework::signer;

        let addr = signer::address_of(account);
        let cap_opt = spec_cap<ConfigValue>(addr);
        // let cap = option::borrow<ConfigValue>(spec_cap<ConfigValue>(signer::address_of(account)));

        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(addr);
        aborts_if option::is_none<ModifyConfigCapability<ConfigValue>>(cap_opt);
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(addr);

        // TODO: For unknown reason we can't specify the strict abort conditions.
        // Intuitively, the commented out spec should be able to be verified because
        // it is exactly the spec of the callee `set_with_capability()`.
        //aborts_if !exists<Config<ConfigValue>>(option::borrow(spec_cap<ConfigValue>(signer::address_of(account))).account_address);
        pragma aborts_if_is_partial;
        ensures exists<Config<ConfigValue>>(
            option::borrow<ModifyConfigCapability<ConfigValue>>(
                spec_cap<ConfigValue>(signer::address_of(account))
            ).account_address,
        );
        ensures global<Config<ConfigValue>>(
            option::borrow<ModifyConfigCapability<ConfigValue>>(
                spec_cap<ConfigValue>(signer::address_of(account))
            ).account_address,
        ).payload == payload;
    }


    spec fun spec_cap<ConfigValue>(addr: address): option::Option<ModifyConfigCapability<ConfigValue>> {
        global<ModifyConfigCapabilityHolder<ConfigValue>>(addr).cap
    }

    spec set_with_capability {
        aborts_if !exists<Config<ConfigValue>>(cap.account_address);
        ensures exists<Config<ConfigValue>>(cap.account_address);
        ensures global<Config<ConfigValue>>(cap.account_address).payload == payload;
    }


    spec publish_new_config_with_capability {
        include PublishNewConfigAbortsIf<ConfigValue>;

        ensures exists<Config<ConfigValue>>(signer::address_of(account));
        ensures global<Config<ConfigValue>>(signer::address_of(account)).payload == payload;

        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer::address_of(account));
        ensures option::is_none(global<ModifyConfigCapabilityHolder<ConfigValue>>(signer::address_of(account)).cap);
    }

    spec publish_new_config {
        include PublishNewConfigAbortsIf<ConfigValue>;

        ensures exists<Config<ConfigValue>>(signer::address_of(account));
        ensures global<Config<ConfigValue>>(signer::address_of(account)).payload == payload;

        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer::address_of(account));
        ensures option::is_some(global<ModifyConfigCapabilityHolder<ConfigValue>>(signer::address_of(account)).cap);
    }

    spec schema PublishNewConfigAbortsIf<ConfigValue> {
        account: signer;
        aborts_if exists<Config<ConfigValue>>(signer::address_of(account));
        aborts_if exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer::address_of(account));
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
        ensures exists<Config<ConfigValue>>(signer::address_of(account));
        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(signer::address_of(account));
    }

    spec schema AbortsIfCapNotExist<ConfigValue> {
        address: address;
        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(address);
        aborts_if option::is_none<ModifyConfigCapability<ConfigValue>>(
            global<ModifyConfigCapabilityHolder<ConfigValue>>(address).cap,
        );
    }

    spec extract_modify_config_capability {
        let address = signer::address_of(account);
        include AbortsIfCapNotExist<ConfigValue>;

        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(address);
        ensures option::is_none<ModifyConfigCapability<ConfigValue>>(
            global<ModifyConfigCapabilityHolder<ConfigValue>>(address).cap
        );
        ensures result == old(option::borrow(global<ModifyConfigCapabilityHolder<ConfigValue>>(address).cap));
    }

    spec restore_modify_config_capability {
        aborts_if !exists<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        aborts_if option::is_some(global<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address).cap);

        ensures exists<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address);
        ensures option::is_some(global<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address).cap);
        ensures option::borrow(global<ModifyConfigCapabilityHolder<ConfigValue>>(cap.account_address).cap) == cap;
    }

    spec destroy_modify_config_capability {
        aborts_if false;
    }

    spec account_address {
        aborts_if false;
        ensures result == cap.account_address;
    }

    spec emit_config_change_event {
        aborts_if false;
    }
}
