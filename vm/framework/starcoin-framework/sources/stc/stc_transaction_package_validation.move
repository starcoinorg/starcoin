/// The module provides strategies for module upgrading.
module starcoin_framework::stc_transaction_package_validation {
    use std::error;
    use std::option;
    use std::signer;
    use starcoin_framework::system_addresses;
    use starcoin_framework::stc_version;

    use starcoin_framework::account;
    use starcoin_framework::event;
    use starcoin_framework::on_chain_config;
    use starcoin_framework::timestamp;


    // /// module upgrade plan
    // struct UpgradePlan has copy, drop, store {
    //     package_hash: vector<u8>,
    //     active_after_time: u64,
    //     version: u64,
    // }

    /// The holder of UpgradePlanCapability for account_address can submit UpgradePlan for account_address.
    struct UpgradePlanCapability has key, store {
        account_address: address,
    }

    const STRATEGY_ARBITRARY: u8 = 0;
    const STRATEGY_TWO_PHASE: u8 = 1;
    const STRATEGY_NEW_MODULE: u8 = 2;
    const STRATEGY_FREEZE: u8 = 3;
    const DEFAULT_MIN_TIME_LIMIT: u64 = 86400000;// one day

    /// arbitary stragegy
    public fun get_strategy_arbitrary(): u8 { STRATEGY_ARBITRARY }

    /// two phase stragegy
    public fun get_strategy_two_phase(): u8 { STRATEGY_TWO_PHASE }

    /// new module strategy
    public fun get_strategy_new_module(): u8 { STRATEGY_NEW_MODULE }

    /// freezed strategy
    public fun get_strategy_freeze(): u8 { STRATEGY_FREEZE }

    /// default min time limit
    public fun get_default_min_time_limit(): u64 { DEFAULT_MIN_TIME_LIMIT }

    const EUPGRADE_PLAN_IS_NONE: u64 = 102;
    const EPACKAGE_HASH_INCORRECT: u64 = 103;
    const EACTIVE_TIME_INCORRECT: u64 = 104;
    const ESTRATEGY_FREEZED: u64 = 105;
    const ESTRATEGY_INCORRECT: u64 = 106;
    const ESTRATEGY_NOT_TWO_PHASE: u64 = 107;
    const EUNKNOWN_STRATEGY: u64 = 108;
    const ESENDER_AND_PACKAGE_ADDRESS_MISMATCH: u64 = 109;

    struct UpgradePlanV2 has copy, drop, store {
        package_hash: vector<u8>,
        active_after_time: u64,
        version: u64,
        enforced: bool,
    }

    /// module upgrade strategy
    struct ModuleUpgradeStrategy has key, store {
        /// 0 arbitrary
        /// 1 two phase upgrade
        /// 2 only new module
        /// 3 freeze
        strategy: u8,
    }

    /// config of two phase upgrade strategy.
    struct TwoPhaseUpgradeConfig has copy, drop, store {
        min_time_limit: u64,
    }

    /// data of two phase upgrade strategy.
    struct TwoPhaseUpgradeV2 has key {
        config: TwoPhaseUpgradeConfig,
        plan: option::Option<UpgradePlanV2>,
        version_cap: on_chain_config::ModifyConfigCapability<stc_version::Version>,
        upgrade_event: event::EventHandle<Self::UpgradeEvent>,
    }

    /// module upgrade event.
    struct UpgradeEvent has drop, store {
        package_address: address,
        package_hash: vector<u8>,
        version: u64,
    }

    /// Update account's ModuleUpgradeStrategy
    public fun update_module_upgrade_strategy(
        account: &signer,
        strategy: u8,
        min_time: option::Option<u64>
    ) acquires ModuleUpgradeStrategy, TwoPhaseUpgradeV2, UpgradePlanCapability {
        assert!(
            strategy == STRATEGY_ARBITRARY || strategy == STRATEGY_TWO_PHASE || strategy == STRATEGY_NEW_MODULE || strategy == STRATEGY_FREEZE,
            error::invalid_argument(EUNKNOWN_STRATEGY)
        );

        let account_address = signer::address_of(account);
        let previous_strategy = get_module_upgrade_strategy(account_address);
        assert!(strategy > previous_strategy, error::invalid_argument(ESTRATEGY_INCORRECT));
        if (exists<ModuleUpgradeStrategy>(account_address)) {
            borrow_global_mut<ModuleUpgradeStrategy>(account_address).strategy = strategy;
        } else {
            move_to(account, ModuleUpgradeStrategy { strategy: strategy });
        };

        if (strategy == STRATEGY_TWO_PHASE) {
            let version_cap = on_chain_config::extract_modify_config_capability<stc_version::Version>(account);
            let min_time_limit = option::get_with_default(&min_time, DEFAULT_MIN_TIME_LIMIT);
            move_to(account, UpgradePlanCapability { account_address: account_address });
            move_to(account, TwoPhaseUpgradeV2 {
                config: TwoPhaseUpgradeConfig { min_time_limit: min_time_limit },
                plan: option::none<UpgradePlanV2>(),
                version_cap: version_cap,
                upgrade_event: account::new_event_handle<Self::UpgradeEvent>(account)
            }
            );
        };

        //clean two phase upgrade resource
        if (previous_strategy == STRATEGY_TWO_PHASE) {
            // if (exists<TwoPhaseUpgrade>(account_address)) {
            //     let tpu = move_from<TwoPhaseUpgrade>(account_address);
            //     let TwoPhaseUpgrade { plan: _, version_cap, upgrade_event, config: _ } = tpu;
            //     event::destroy_handle<Self::UpgradeEvent>(upgrade_event);
            //     on_chain_config::destroy_modify_config_capability<stc_version::Version>(version_cap);
            // };
            if (exists<TwoPhaseUpgradeV2>(account_address)) {
                let tpu = move_from<TwoPhaseUpgradeV2>(account_address);
                let TwoPhaseUpgradeV2 { plan: _, version_cap, upgrade_event, config: _ } = tpu;
                event::destroy_handle<Self::UpgradeEvent>(upgrade_event);
                on_chain_config::destroy_modify_config_capability<stc_version::Version>(version_cap);
            };
            // UpgradePlanCapability may be extracted
            if (exists<UpgradePlanCapability>(account_address)) {
                let cap = move_from<UpgradePlanCapability>(account_address);
                destroy_upgrade_plan_cap(cap);
            };
        };
    }


    /// Get account address of UpgradePlanCapability
    public fun account_address(cap: &UpgradePlanCapability): address {
        cap.account_address
    }

    /// destroy the given UpgradePlanCapability
    public fun destroy_upgrade_plan_cap(cap: UpgradePlanCapability) {
        let UpgradePlanCapability { account_address: _ } = cap;
    }


    /// extract out UpgradePlanCapability from `signer`.
    public fun extract_submit_upgrade_plan_cap(
        account: &signer
    ): UpgradePlanCapability acquires ModuleUpgradeStrategy, UpgradePlanCapability {
        let account_address = signer::address_of(account);
        assert!(
            get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE,
            error::invalid_argument(ESTRATEGY_NOT_TWO_PHASE)
        );
        move_from<UpgradePlanCapability>(account_address)
    }

    // public entry fun convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2(
    //     account: signer,
    //     package_address: address
    // ) acquires TwoPhaseUpgrade {
    //     let account_address = signer::address_of(&account);
    //     // sender should be package owner
    //     assert!(account_address == package_address, error::not_found(ESENDER_AND_PACKAGE_ADDRESS_MISMATCH));
    //     let tpu = move_from<TwoPhaseUpgrade>(account_address);
    //     let TwoPhaseUpgrade { config, plan, version_cap, upgrade_event } = tpu;
    //     if (option::is_some(&plan)) {
    //         let old_plan = option::borrow(&plan);
    //         move_to(&account, TwoPhaseUpgradeV2 {
    //             config: config,
    //             plan: option::some(UpgradePlanV2 {
    //                 package_hash: *&old_plan.package_hash,
    //                 active_after_time: old_plan.active_after_time,
    //                 version: old_plan.version,
    //                 enforced: false
    //             }),
    //             version_cap: version_cap,
    //             upgrade_event: upgrade_event
    //         });
    //     } else {
    //         move_to(&account, TwoPhaseUpgradeV2 {
    //             config: config,
    //             plan: option::none<UpgradePlanV2>(),
    //             version_cap: version_cap,
    //             upgrade_event: upgrade_event
    //         });
    //     };
    // }

    public fun submit_upgrade_plan_v2(
        account: &signer,
        package_hash: vector<u8>,
        version: u64,
        enforced: bool
    ) acquires TwoPhaseUpgradeV2, UpgradePlanCapability, ModuleUpgradeStrategy {
        let account_address = signer::address_of(account);
        let cap = borrow_global<UpgradePlanCapability>(account_address);
        submit_upgrade_plan_with_cap_v2(cap, package_hash, version, enforced);
    }

    public fun submit_upgrade_plan_with_cap_v2(
        cap: &UpgradePlanCapability,
        package_hash: vector<u8>,
        version: u64,
        enforced: bool
    ) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
        let package_address = cap.account_address;
        assert!(
            get_module_upgrade_strategy(package_address) == STRATEGY_TWO_PHASE,
            error::invalid_argument(ESTRATEGY_NOT_TWO_PHASE)
        );
        let tpu = borrow_global_mut<TwoPhaseUpgradeV2>(package_address);
        let active_after_time = timestamp::now_milliseconds() + tpu.config.min_time_limit;
        tpu.plan = option::some(UpgradePlanV2 { package_hash, active_after_time, version, enforced });
    }


    /// Cancel a module upgrade plan.
    public fun cancel_upgrade_plan(
        account: &signer
    ) acquires TwoPhaseUpgradeV2, UpgradePlanCapability, ModuleUpgradeStrategy {
        let account_address = signer::address_of(account);
        let cap = borrow_global<UpgradePlanCapability>(account_address);
        cancel_upgrade_plan_with_cap(cap);
    }


    /// Cancel a module upgrade plan with given cap.
    public fun cancel_upgrade_plan_with_cap(
        cap: &UpgradePlanCapability
    ) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
        let package_address = cap.account_address;
        assert!(
            get_module_upgrade_strategy(package_address) == STRATEGY_TWO_PHASE,
            error::invalid_argument(ESTRATEGY_NOT_TWO_PHASE)
        );
        let tpu = borrow_global_mut<TwoPhaseUpgradeV2>(package_address);
        assert!(option::is_some(&tpu.plan), error::invalid_state(EUPGRADE_PLAN_IS_NONE));
        tpu.plan = option::none<UpgradePlanV2>();
    }


    /// Get module upgrade strategy of an module address.
    public fun get_module_upgrade_strategy(module_address: address): u8 acquires ModuleUpgradeStrategy {
        if (exists<ModuleUpgradeStrategy>(module_address)) {
            borrow_global<ModuleUpgradeStrategy>(module_address).strategy
        }else {
            0
        }
    }

    // /// Get module upgrade plan of an address.
    // public fun get_upgrade_plan(_module_address: address): option::Option<UpgradePlan> {
    //     // DEPRECATED_CODE
    //     option::none<UpgradePlan>()
    // }

    /// Get module upgrade plan of an address.
    public fun get_upgrade_plan_v2(module_address: address): option::Option<UpgradePlanV2> acquires TwoPhaseUpgradeV2 {
        if (exists<TwoPhaseUpgradeV2>(module_address)) {
            *&borrow_global<TwoPhaseUpgradeV2>(module_address).plan
        } else {
            option::none<UpgradePlanV2>()
        }
    }


    /// Check againest on the given package data.
    public fun check_package_txn(
        package_address: address,
        package_hash: vector<u8>
    ) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
        let strategy = get_module_upgrade_strategy(package_address);
        if (strategy == STRATEGY_ARBITRARY) {
            //do nothing
        }else if (strategy == STRATEGY_TWO_PHASE) {
            let plan_opt = get_upgrade_plan_v2(package_address);
            assert!(option::is_some(&plan_opt), error::invalid_argument(EUPGRADE_PLAN_IS_NONE));
            let plan = option::borrow(&plan_opt);
            assert!(*&plan.package_hash == package_hash, error::invalid_argument(EPACKAGE_HASH_INCORRECT));
            assert!(
                plan.active_after_time <= timestamp::now_milliseconds(),
                error::invalid_argument(EACTIVE_TIME_INCORRECT)
            );
        }else if (strategy == STRATEGY_NEW_MODULE) {
            //do check at VM runtime.
        }else if (strategy == STRATEGY_FREEZE) {
            error::invalid_argument(ESTRATEGY_FREEZED);
        };
    }

    public fun check_package_txn_v2(
        txn_sender: address,
        package_address: address,
        package_hash: vector<u8>
    ) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
        let strategy = get_module_upgrade_strategy(package_address);
        if (strategy == STRATEGY_ARBITRARY) {
            assert!(txn_sender == package_address, error::not_found(ESENDER_AND_PACKAGE_ADDRESS_MISMATCH));
        }else if (strategy == STRATEGY_TWO_PHASE) {
            let plan_opt = get_upgrade_plan_v2(package_address);
            assert!(option::is_some(&plan_opt), error::invalid_argument(EUPGRADE_PLAN_IS_NONE));
            let plan = option::borrow<UpgradePlanV2>(&plan_opt);
            assert!(*&plan.package_hash == package_hash, error::invalid_argument(EPACKAGE_HASH_INCORRECT));
            assert!(
                plan.active_after_time <= timestamp::now_milliseconds(),
                error::invalid_argument(EACTIVE_TIME_INCORRECT)
            );
        }else if (strategy == STRATEGY_NEW_MODULE) {
            //do check at VM runtime.
            assert!(txn_sender == package_address, error::not_found(ESENDER_AND_PACKAGE_ADDRESS_MISMATCH));
        }else if (strategy == STRATEGY_FREEZE) {
            error::invalid_argument(ESTRATEGY_FREEZED);
        };
    }


    fun finish_upgrade_plan(package_address: address) acquires TwoPhaseUpgradeV2 {
        let tpu = borrow_global_mut<TwoPhaseUpgradeV2>(package_address);
        if (option::is_some(&tpu.plan)) {
            let plan = option::borrow(&tpu.plan);
            on_chain_config::set_with_capability<stc_version::Version>(
                &mut tpu.version_cap,
                stc_version::new_version(plan.version)
            );
            event::emit_event<Self::UpgradeEvent>(&mut tpu.upgrade_event, UpgradeEvent {
                package_address,
                package_hash: *&plan.package_hash,
                version: plan.version
            });
        };
        tpu.plan = option::none<UpgradePlanV2>();
    }

    // /// Prologue of package transaction.
    // public fun package_txn_prologue(
    //     account: &signer,
    //     package_address: address,
    //     package_hash: vector<u8>
    // ) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
    //     // Can only be invoked by genesis account
    //     system_addresses::assert_starcoin_framework(account);
    //     check_package_txn(package_address, package_hash);
    // }

    public fun package_txn_prologue_v2(
        account: &signer,
        txn_sender: address,
        package_address: address,
        package_hash: vector<u8>
    ) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
        // Can only be invoked by genesis account
        system_addresses::assert_starcoin_framework(account);
        check_package_txn_v2(txn_sender, package_address, package_hash);
    }

    /// Package txn finished, and clean UpgradePlan
    public fun package_txn_epilogue(
        account: &signer,
        _txn_sender: address,
        package_address: address,
        success: bool
    ) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
        // Can only be invoked by genesis account
        system_addresses::assert_starcoin_framework(account);
        let strategy = get_module_upgrade_strategy(package_address);
        if (strategy == STRATEGY_TWO_PHASE) {
            if (success) {
                finish_upgrade_plan(package_address);
            };
        };
    }
}