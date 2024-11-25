/// `VMConfig` keep track of VM related configuration, like gas schedule.
module starcoin_framework::vm_config {
    use starcoin_framework::storage_gas::GasCurve;
    use starcoin_framework::gas_schedule::GasScheduleV2;
    use starcoin_framework::system_addresses;
    use starcoin_framework::on_chain_config;
    use starcoin_framework::util;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict;
    }

    /// The struct to hold all config data needed to operate the VM.
    /// * gas_schedule: Cost of running the VM.
    struct VMConfig has copy, drop, store {
        gas_schedule: GasScheduleV2,
    }

    /// Initialize the table under the genesis account
    public fun initialize(
        account: &signer,
        gas_schedule_blob: vector<u8>,
    ) {
        // CoreAddresses::assert_genesis_address(account);
        system_addresses::assert_starcoin_framework(account);
        let gas_schedule: GasScheduleV2 = util::from_bytes(gas_schedule_blob);
        on_chain_config::publish_new_config<VMConfig>(
            account,
            VMConfig {
                gas_schedule,
            },
        );
    }

    public fun new_from_blob(gas_schedule_blob: vector<u8>): VMConfig {
        util::from_bytes<VMConfig>(gas_schedule_blob)
    }

    spec initialize {
        use std::signer;
        use starcoin_framework::on_chain_config;

        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<on_chain_config::Config<VMConfig>>(signer::address_of(account));
        aborts_if
            exists<on_chain_config::ModifyConfigCapabilityHolder<VMConfig>>(
                signer::address_of(account),
            );
        ensures exists<on_chain_config::Config<VMConfig>>(signer::address_of(account));
        ensures
            exists<on_chain_config::ModifyConfigCapabilityHolder<VMConfig>>(
                signer::address_of(account),
            );
    }
}