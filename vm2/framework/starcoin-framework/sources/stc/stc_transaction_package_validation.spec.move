/// The module provides strategies for module upgrading.
spec starcoin_framework::stc_transaction_package_validation {
    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    spec update_module_upgrade_strategy {
        pragma verify = false;
        aborts_if strategy != 0
            && strategy != 1
            && strategy != 2
            && strategy != 3;
        aborts_if exists<ModuleUpgradeStrategy>(signer::address_of(account))
            && strategy <= global<ModuleUpgradeStrategy>(signer::address_of(account)).strategy;
        aborts_if !exists<ModuleUpgradeStrategy>(signer::address_of(account)) && strategy == 0;

        aborts_if strategy == 1 && exists<UpgradePlanCapability>(signer::address_of(account));
        aborts_if strategy == 1 && !exists<on_chain_config::ModifyConfigCapabilityHolder<stc_version::Version>>(
            signer::address_of(account)
        );
        let holder = global<on_chain_config::ModifyConfigCapabilityHolder<stc_version::Version>>(
            signer::address_of(account)
        );
        aborts_if strategy == 1 && option::is_none<on_chain_config::ModifyConfigCapability<stc_version::Version>>(
            holder.cap
        );
        aborts_if strategy == 1 && exists<TwoPhaseUpgradeV2>(signer::address_of(account));

        aborts_if exists<ModuleUpgradeStrategy>(signer::address_of(account)) && global<ModuleUpgradeStrategy>(
            signer::address_of(account)
        ).strategy == 1
            && !exists<TwoPhaseUpgradeV2>(signer::address_of(account));
    }


    spec destroy_upgrade_plan_cap {
        aborts_if false;
    }


    spec extract_submit_upgrade_plan_cap {
        aborts_if !exists<ModuleUpgradeStrategy>(signer::address_of(account));
        aborts_if global<ModuleUpgradeStrategy>(signer::address_of(account)).strategy != 1;
        aborts_if !exists<UpgradePlanCapability>(signer::address_of(account));
    }

    spec submit_upgrade_plan_v2 {
        pragma verify = false;

        aborts_if !exists<UpgradePlanCapability>(signer::address_of(account));
        include SubmitUpgradePlanWithCapAbortsIf {
            account: global<UpgradePlanCapability>(signer::address_of(account)).account_address
        };
        ensures option::is_some(
            global<TwoPhaseUpgradeV2>(global<UpgradePlanCapability>(signer::address_of(account)).account_address).plan
        );
    }

    spec submit_upgrade_plan_with_cap_v2 {
        pragma verify = false;
        include SubmitUpgradePlanWithCapAbortsIf { account: cap.account_address };
        ensures option::is_some(global<TwoPhaseUpgradeV2>(cap.account_address).plan);
    }

    spec schema SubmitUpgradePlanWithCapAbortsIf {
        account: address;
        aborts_if !exists<ModuleUpgradeStrategy>(account);
        aborts_if global<ModuleUpgradeStrategy>(account).strategy != 1;
        aborts_if !exists<TwoPhaseUpgradeV2>(account);
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if timestamp::now_milliseconds() + global<TwoPhaseUpgradeV2>(account).config.min_time_limit > max_u64();
    }


    spec cancel_upgrade_plan {
        aborts_if !exists<UpgradePlanCapability>(signer::address_of(account));
        include CancelUpgradePlanWithCapAbortsIf {
            account: global<UpgradePlanCapability>(
                signer::address_of(account)
            ).account_address
        };
        ensures option::is_none(
            global<TwoPhaseUpgradeV2>(global<UpgradePlanCapability>(signer::address_of(account)).account_address).plan
        );
    }


    spec cancel_upgrade_plan_with_cap {
        include CancelUpgradePlanWithCapAbortsIf { account: cap.account_address };
        ensures option::is_none(global<TwoPhaseUpgradeV2>(cap.account_address).plan);
    }

    spec schema CancelUpgradePlanWithCapAbortsIf {
        account: address;
        aborts_if !exists<ModuleUpgradeStrategy>(account);
        aborts_if global<ModuleUpgradeStrategy>(account).strategy != 1;
        aborts_if !exists<TwoPhaseUpgradeV2>(account);
        aborts_if !option::is_some(global<TwoPhaseUpgradeV2>(account).plan);
    }

    spec get_module_upgrade_strategy {
        aborts_if false;
    }

    spec fun spec_get_module_upgrade_strategy(module_address: address): u8 {
        if (exists<ModuleUpgradeStrategy>(module_address)) {
            global<ModuleUpgradeStrategy>(module_address).strategy
        } else {
            0
        }
    }


    // spec get_upgrade_plan {
    //     aborts_if false;
    // }

    spec get_upgrade_plan_v2 {
        pragma verify = false;
        aborts_if false;
    }

    spec fun spec_get_upgrade_plan_v2(module_address: address): option::Option<UpgradePlanV2> {
        if (exists<TwoPhaseUpgradeV2>(module_address)) {
            global<TwoPhaseUpgradeV2>(module_address).plan
        }else {
            option::spec_none<UpgradePlanV2>()
        }
    }


    spec check_package_txn {
        pragma verify = false;
        include CheckPackageTxnAbortsIf;
    }

    spec schema CheckPackageTxnAbortsIf {
        package_address: address;
        package_hash: vector<u8>;

        aborts_if spec_get_module_upgrade_strategy(package_address) == 3;
        aborts_if spec_get_module_upgrade_strategy(package_address) == 1
            && option::is_none(spec_get_upgrade_plan_v2(package_address));
        aborts_if spec_get_module_upgrade_strategy(package_address) == 1
            && option::borrow(spec_get_upgrade_plan_v2(package_address)).package_hash != package_hash;
        aborts_if spec_get_module_upgrade_strategy(package_address) == 1
            && !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if spec_get_module_upgrade_strategy(package_address) == 1
            && option::borrow(spec_get_upgrade_plan_v2(package_address)).active_after_time > timestamp::now_milliseconds();
    }

    spec schema CheckPackageTxnAbortsIfWithType {
        is_package: bool;
        sender: address;
        package_address: address;
        package_hash: vector<u8>;
        aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 3;
        aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && option::is_none(
            spec_get_upgrade_plan_v2(package_address)
        );
        aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && option::borrow(
            spec_get_upgrade_plan_v2(package_address)
        ).package_hash != package_hash;
        aborts_if is_package && spec_get_module_upgrade_strategy(
            package_address
        ) == 1 && !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && option::borrow(
            spec_get_upgrade_plan_v2(package_address)
        ).active_after_time > timestamp::now_milliseconds();
    }


    spec finish_upgrade_plan {
        pragma verify = false;
        aborts_if !exists<TwoPhaseUpgradeV2>(package_address);
        let tpu = global<TwoPhaseUpgradeV2>(package_address);
        aborts_if option::is_some(tpu.plan) && !exists<on_chain_config::Config<stc_version::Version>>(
            tpu.version_cap.account_address
        );
    }


    // spec package_txn_prologue {
    //     aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
    //     include CheckPackageTxnAbortsIf {};
    // }

    spec schema AbortsIfPackageTxnEpilogue {
        is_package: bool;
        package_address: address;
        success: bool;

        aborts_if is_package
            && get_module_upgrade_strategy(package_address) == STRATEGY_TWO_PHASE
            && success
            && !exists<TwoPhaseUpgradeV2>(package_address);
    }

    spec package_txn_epilogue {
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if spec_get_module_upgrade_strategy(package_address) == 1
            && success
            && !exists<TwoPhaseUpgradeV2>(package_address);
        aborts_if spec_get_module_upgrade_strategy(package_address) == 1
            && success
            && option::is_some(global<TwoPhaseUpgradeV2>(package_address).plan)
            && !exists<on_chain_config::Config<stc_version::Version>>(
            global<TwoPhaseUpgradeV2>(package_address).version_cap.account_address
        );
    }
}