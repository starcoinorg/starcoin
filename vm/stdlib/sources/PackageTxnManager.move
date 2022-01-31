address StarcoinFramework {
    /// The module provides strategies for module upgrading.
    module PackageTxnManager {
        use StarcoinFramework::Option::{Self,Option};
        use StarcoinFramework::Signer;
        use StarcoinFramework::CoreAddresses;
        use StarcoinFramework::Errors;
        use StarcoinFramework::Version;
        use StarcoinFramework::Event;
        use StarcoinFramework::Config;
        use StarcoinFramework::Timestamp;

        spec module {
            pragma verify = false;
            pragma aborts_if_is_strict = true;
        }
        /// module upgrade plan
        struct UpgradePlan has copy, drop, store {
            package_hash: vector<u8>,
            active_after_time: u64,
            version: u64,
        }

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

        /// data of two phase upgrade strategy.
        struct TwoPhaseUpgrade has key {
            config: TwoPhaseUpgradeConfig,
            plan: Option<UpgradePlan>,
            version_cap: Config::ModifyConfigCapability<Version::Version>,
            upgrade_event: Event::EventHandle<Self::UpgradeEvent>,
        }

        /// config of two phase upgrade strategy.
        struct TwoPhaseUpgradeConfig has copy, drop, store {
            min_time_limit: u64,
        }

        /// data of two phase upgrade strategy.
        struct TwoPhaseUpgradeV2 has key {
            config: TwoPhaseUpgradeConfig,
            plan: Option<UpgradePlanV2>,
            version_cap: Config::ModifyConfigCapability<Version::Version>,
            upgrade_event: Event::EventHandle<Self::UpgradeEvent>,
        }

        /// module upgrade event.
        struct UpgradeEvent has drop, store {
            package_address: address,
            package_hash: vector<u8>,
            version: u64,
        }

        /// Update account's ModuleUpgradeStrategy
        public fun update_module_upgrade_strategy(account: &signer, strategy: u8, min_time: Option<u64>) acquires ModuleUpgradeStrategy, TwoPhaseUpgrade, TwoPhaseUpgradeV2, UpgradePlanCapability{
            assert!(strategy == STRATEGY_ARBITRARY || strategy == STRATEGY_TWO_PHASE || strategy == STRATEGY_NEW_MODULE || strategy == STRATEGY_FREEZE, Errors::invalid_argument(EUNKNOWN_STRATEGY));
            let account_address = Signer::address_of(account);
            let previous_strategy = get_module_upgrade_strategy(account_address);
            assert!(strategy > previous_strategy, Errors::invalid_argument(ESTRATEGY_INCORRECT));
            if (exists<ModuleUpgradeStrategy>(account_address)) {
                borrow_global_mut<ModuleUpgradeStrategy>(account_address).strategy = strategy;
            }else{
                move_to(account, ModuleUpgradeStrategy{ strategy: strategy});
            };
            if (strategy == STRATEGY_TWO_PHASE){
                let version_cap = Config::extract_modify_config_capability<Version::Version>(account);
                let min_time_limit = Option::get_with_default(&min_time, DEFAULT_MIN_TIME_LIMIT);
                move_to(account, UpgradePlanCapability{ account_address: account_address});
                move_to(account, TwoPhaseUpgradeV2{
                    config: TwoPhaseUpgradeConfig{min_time_limit: min_time_limit},
                    plan: Option::none<UpgradePlanV2>(),
                    version_cap: version_cap,
                    upgrade_event: Event::new_event_handle<Self::UpgradeEvent>(account)}
                );
            };
            //clean two phase upgrade resource
            if (previous_strategy == STRATEGY_TWO_PHASE){
                if (exists<TwoPhaseUpgrade>(account_address)) {
                    let tpu = move_from<TwoPhaseUpgrade>(account_address);
                    let TwoPhaseUpgrade{plan:_, version_cap, upgrade_event, config: _} = tpu;
                    Event::destroy_handle<Self::UpgradeEvent>(upgrade_event);
                    Config::destroy_modify_config_capability<Version::Version>(version_cap);
                };
                if (exists<TwoPhaseUpgradeV2>(account_address)) {
                    let tpu = move_from<TwoPhaseUpgradeV2>(account_address);
                    let TwoPhaseUpgradeV2{plan:_, version_cap, upgrade_event, config: _} = tpu;
                    Event::destroy_handle<Self::UpgradeEvent>(upgrade_event);
                    Config::destroy_modify_config_capability<Version::Version>(version_cap);
                };
                // UpgradePlanCapability may be extracted
                if (exists<UpgradePlanCapability>(account_address)) {
                    let cap = move_from<UpgradePlanCapability>(account_address);
                    destroy_upgrade_plan_cap(cap);
                };
            };
        }

        spec update_module_upgrade_strategy {
            pragma verify = false;
            aborts_if strategy != 0 && strategy != 1 && strategy != 2 && strategy != 3;
            aborts_if exists<ModuleUpgradeStrategy>(Signer::address_of(account)) && strategy <= global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy;
            aborts_if !exists<ModuleUpgradeStrategy>(Signer::address_of(account)) && strategy == 0;

            aborts_if strategy == 1 && exists<UpgradePlanCapability>(Signer::address_of(account));
            aborts_if strategy == 1 && !exists<Config::ModifyConfigCapabilityHolder<Version::Version>>(Signer::address_of(account));
            let holder = global<Config::ModifyConfigCapabilityHolder<Version::Version>>(Signer::address_of(account));
            aborts_if strategy == 1 && Option::is_none<Config::ModifyConfigCapability<Version::Version>>(holder.cap);
            aborts_if strategy == 1 && exists<TwoPhaseUpgrade>(Signer::address_of(account));

            aborts_if exists<ModuleUpgradeStrategy>(Signer::address_of(account)) && global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy == 1
                && !exists<TwoPhaseUpgrade>(Signer::address_of(account));
        }

        /// Get account address of UpgradePlanCapability
        public fun account_address(cap: &UpgradePlanCapability): address {
            cap.account_address
        }

        /// destroy the given UpgradePlanCapability
        public fun destroy_upgrade_plan_cap(cap: UpgradePlanCapability){
            let UpgradePlanCapability{account_address:_} = cap;
        }

        spec destroy_upgrade_plan_cap {
            aborts_if false;
        }

        /// extract out UpgradePlanCapability from `signer`.
        public fun extract_submit_upgrade_plan_cap(account: &signer): UpgradePlanCapability acquires ModuleUpgradeStrategy, UpgradePlanCapability{
            let account_address = Signer::address_of(account);
            assert!(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE, Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            move_from<UpgradePlanCapability>(account_address)
        }

        spec extract_submit_upgrade_plan_cap {
            aborts_if !exists<ModuleUpgradeStrategy>(Signer::address_of(account));
            aborts_if global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy != 1;
            aborts_if !exists<UpgradePlanCapability>(Signer::address_of(account));
        }

        public(script) fun convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2(account: signer, package_address: address) acquires TwoPhaseUpgrade {
            let account_address = Signer::address_of(&account);
            // sender should be package owner
            assert!(account_address == package_address, Errors::requires_address(ESENDER_AND_PACKAGE_ADDRESS_MISMATCH));
            let tpu = move_from<TwoPhaseUpgrade>(account_address);
            let TwoPhaseUpgrade{config, plan, version_cap, upgrade_event} = tpu;
            if (Option::is_some(&plan)) {
                let old_plan = Option::borrow(&plan);
                move_to(&account, TwoPhaseUpgradeV2{
                    config: config,
                    plan: Option::some(UpgradePlanV2 {
                        package_hash: *&old_plan.package_hash,
                        active_after_time: old_plan.active_after_time,
                        version: old_plan.version,
                        enforced: false }),
                    version_cap: version_cap,
                    upgrade_event: upgrade_event
                });
            } else {
                move_to(&account, TwoPhaseUpgradeV2{
                    config: config,
                    plan: Option::none<UpgradePlanV2>(),
                    version_cap: version_cap,
                    upgrade_event: upgrade_event
                });
            };
        }

        spec convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2 {
            pragma verify = false;
        }

        public fun submit_upgrade_plan_v2(account: &signer, package_hash: vector<u8>, version:u64, enforced: bool) acquires TwoPhaseUpgradeV2,UpgradePlanCapability,ModuleUpgradeStrategy{
            let account_address = Signer::address_of(account);
            let cap = borrow_global<UpgradePlanCapability>(account_address);
            submit_upgrade_plan_with_cap_v2(cap, package_hash, version, enforced);
        }

        spec submit_upgrade_plan_v2 {
            pragma verify = false;
            aborts_if !exists<UpgradePlanCapability>(Signer::address_of(account));
            include SubmitUpgradePlanWithCapAbortsIf{account: global<UpgradePlanCapability>(Signer::address_of(account)).account_address};
            ensures Option::is_some(global<TwoPhaseUpgrade>(global<UpgradePlanCapability>(Signer::address_of(account)).account_address).plan);
        }
        public fun submit_upgrade_plan_with_cap_v2(cap: &UpgradePlanCapability, package_hash: vector<u8>, version: u64, enforced: bool) acquires TwoPhaseUpgradeV2,ModuleUpgradeStrategy{
            let package_address = cap.account_address;
            assert!(get_module_upgrade_strategy(package_address) == STRATEGY_TWO_PHASE, Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            let tpu = borrow_global_mut<TwoPhaseUpgradeV2>(package_address);
            let active_after_time = Timestamp::now_milliseconds() + tpu.config.min_time_limit;
            tpu.plan = Option::some(UpgradePlanV2 { package_hash, active_after_time, version, enforced });
        }
        spec submit_upgrade_plan_with_cap_v2 {
            pragma verify = false;
            include SubmitUpgradePlanWithCapAbortsIf{account: cap.account_address};
            ensures Option::is_some(global<TwoPhaseUpgrade>(cap.account_address).plan);
        }

        spec schema SubmitUpgradePlanWithCapAbortsIf {
            account: address;
            aborts_if !exists<ModuleUpgradeStrategy>(account);
            aborts_if global<ModuleUpgradeStrategy>(account).strategy != 1;
            aborts_if !exists<TwoPhaseUpgrade>(account);
            aborts_if !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS());
            aborts_if Timestamp::now_milliseconds() + global<TwoPhaseUpgrade>(account).config.min_time_limit > max_u64();
        }

        /// Cancel a module upgrade plan.
        public fun cancel_upgrade_plan(account: &signer) acquires TwoPhaseUpgradeV2,UpgradePlanCapability,ModuleUpgradeStrategy{
            let account_address = Signer::address_of(account);
            let cap = borrow_global<UpgradePlanCapability>(account_address);
            cancel_upgrade_plan_with_cap(cap);
        }

        spec cancel_upgrade_plan {
            aborts_if !exists<UpgradePlanCapability>(Signer::address_of(account));
            include CancelUpgradePlanWithCapAbortsIf{account: global<UpgradePlanCapability>(Signer::address_of(account)).account_address};
            ensures Option::is_none(global<TwoPhaseUpgrade>(global<UpgradePlanCapability>(Signer::address_of(account)).account_address).plan);
        }

        /// Cancel a module upgrade plan with given cap.
        public fun cancel_upgrade_plan_with_cap(cap: &UpgradePlanCapability) acquires TwoPhaseUpgradeV2,ModuleUpgradeStrategy{
            let package_address = cap.account_address;
            assert!(get_module_upgrade_strategy(package_address) == STRATEGY_TWO_PHASE, Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            let tpu = borrow_global_mut<TwoPhaseUpgradeV2>(package_address);
            assert!(Option::is_some(&tpu.plan), Errors::invalid_state(EUPGRADE_PLAN_IS_NONE));
            tpu.plan = Option::none<UpgradePlanV2>();
        }

        spec cancel_upgrade_plan_with_cap {
            include CancelUpgradePlanWithCapAbortsIf{account: cap.account_address};
            ensures Option::is_none(global<TwoPhaseUpgrade>(cap.account_address).plan);
        }

        spec schema CancelUpgradePlanWithCapAbortsIf {
            account: address;
            aborts_if !exists<ModuleUpgradeStrategy>(account);
            aborts_if global<ModuleUpgradeStrategy>(account).strategy != 1;
            aborts_if !exists<TwoPhaseUpgrade>(account);
            aborts_if !Option::is_some(global<TwoPhaseUpgrade>(account).plan);
        }

        /// Get module upgrade strategy of an module address.
        public fun get_module_upgrade_strategy(module_address: address): u8 acquires ModuleUpgradeStrategy {
            if (exists<ModuleUpgradeStrategy>(module_address)) {
                borrow_global<ModuleUpgradeStrategy>(module_address).strategy
            }else{
                0
            }
        }

        spec get_module_upgrade_strategy {
            aborts_if false;
        }

        spec fun spec_get_module_upgrade_strategy(module_address: address): u8 {
            if (exists<ModuleUpgradeStrategy>(module_address)) {
                global<ModuleUpgradeStrategy>(module_address).strategy
            }else{
                0
            }
        }

        /// Get module upgrade plan of an address.
        public fun get_upgrade_plan(_module_address: address): Option<UpgradePlan> {
            // DEPRECATED_CODE
            Option::none<UpgradePlan>()
        }

        spec get_upgrade_plan {
            aborts_if false;
        }

        /// Get module upgrade plan of an address.
        public fun get_upgrade_plan_v2(module_address: address): Option<UpgradePlanV2> acquires TwoPhaseUpgradeV2 {
            if (exists<TwoPhaseUpgradeV2>(module_address)) {
                *&borrow_global<TwoPhaseUpgradeV2>(module_address).plan
            } else {
                Option::none<UpgradePlanV2>()
            }
        }

        spec get_upgrade_plan_v2 {
            pragma verify = false;
            aborts_if false;
        }
        spec fun spec_get_upgrade_plan_v2(module_address: address): Option<UpgradePlan> {
            if (exists<TwoPhaseUpgrade>(module_address)) {
                global<TwoPhaseUpgrade>(module_address).plan
            }else{
                Option::spec_none<UpgradePlan>()
            }
        }

        /// Check againest on the given package data.
        public fun check_package_txn(package_address: address, package_hash: vector<u8>) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy{
            let strategy = get_module_upgrade_strategy(package_address);
            if (strategy == STRATEGY_ARBITRARY){
                //do nothing
            }else if(strategy == STRATEGY_TWO_PHASE){
                let plan_opt = get_upgrade_plan_v2(package_address);
                assert!(Option::is_some(&plan_opt), Errors::invalid_argument(EUPGRADE_PLAN_IS_NONE));
                let plan = Option::borrow(&plan_opt);
                assert!(*&plan.package_hash == package_hash, Errors::invalid_argument(EPACKAGE_HASH_INCORRECT));
                assert!(plan.active_after_time <= Timestamp::now_milliseconds(), Errors::invalid_argument(EACTIVE_TIME_INCORRECT));
            }else if(strategy == STRATEGY_NEW_MODULE){
                //do check at VM runtime.
            }else if(strategy == STRATEGY_FREEZE){
                abort(ESTRATEGY_FREEZED)
            };
        }

        spec check_package_txn {
            pragma verify = false;
            include CheckPackageTxnAbortsIf;
        }

        public fun check_package_txn_v2(txn_sender: address, package_address: address, package_hash: vector<u8>) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy{
            let strategy = get_module_upgrade_strategy(package_address);
            if (strategy == STRATEGY_ARBITRARY){
                assert!(txn_sender == package_address, Errors::requires_address(ESENDER_AND_PACKAGE_ADDRESS_MISMATCH));
            }else if(strategy == STRATEGY_TWO_PHASE){
                let plan_opt = get_upgrade_plan_v2(package_address);
                assert!(Option::is_some(&plan_opt), Errors::invalid_argument(EUPGRADE_PLAN_IS_NONE));
                let plan = Option::borrow(&plan_opt);
                assert!(*&plan.package_hash == package_hash, Errors::invalid_argument(EPACKAGE_HASH_INCORRECT));
                assert!(plan.active_after_time <= Timestamp::now_milliseconds(), Errors::invalid_argument(EACTIVE_TIME_INCORRECT));
            }else if(strategy == STRATEGY_NEW_MODULE){
                //do check at VM runtime.
                assert!(txn_sender == package_address, Errors::requires_address(ESENDER_AND_PACKAGE_ADDRESS_MISMATCH));
            }else if(strategy == STRATEGY_FREEZE){
                abort(ESTRATEGY_FREEZED)
            };
        }

        spec schema CheckPackageTxnAbortsIf {
            package_address: address;
            package_hash: vector<u8>;
            aborts_if spec_get_module_upgrade_strategy(package_address) == 3;
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1 && Option::is_none(spec_get_upgrade_plan_v2(package_address));
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1 && Option::borrow(spec_get_upgrade_plan_v2(package_address)).package_hash != package_hash;
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1 && !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS());
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1 && Option::borrow(spec_get_upgrade_plan_v2(package_address)).active_after_time > Timestamp::now_milliseconds();
        }

        spec schema CheckPackageTxnAbortsIfWithType {
            is_package: bool;
            sender: address;
            package_address: address;
            package_hash: vector<u8>;
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 3;
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && Option::is_none(spec_get_upgrade_plan_v2(package_address));
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && Option::borrow(spec_get_upgrade_plan_v2(package_address)).package_hash != package_hash;
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && !exists<Timestamp::CurrentTimeMilliseconds>(CoreAddresses::GENESIS_ADDRESS());
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && Option::borrow(spec_get_upgrade_plan_v2(package_address)).active_after_time > Timestamp::now_milliseconds();
        }

        fun finish_upgrade_plan(package_address: address) acquires TwoPhaseUpgradeV2 {
            let tpu = borrow_global_mut<TwoPhaseUpgradeV2>(package_address);
            if (Option::is_some(&tpu.plan)) {
                let plan = Option::borrow(&tpu.plan);
                Config::set_with_capability<Version::Version>(&mut tpu.version_cap, Version::new_version(plan.version));
                Event::emit_event<Self::UpgradeEvent>(&mut tpu.upgrade_event, UpgradeEvent {
                    package_address: package_address,
                    package_hash: *&plan.package_hash,
                    version: plan.version});
            };
            tpu.plan = Option::none<UpgradePlanV2>();
        }

        spec finish_upgrade_plan {
            pragma verify = false;
            aborts_if !exists<TwoPhaseUpgrade>(package_address);
            let tpu = global<TwoPhaseUpgrade>(package_address);
            aborts_if Option::is_some(tpu.plan) && !exists<Config::Config<Version::Version>>(tpu.version_cap.account_address);
        }

        /// Prologue of package transaction.
        public fun package_txn_prologue(account: &signer, package_address: address, package_hash: vector<u8>) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
            // Can only be invoked by genesis account
            CoreAddresses::assert_genesis_address(account);
            check_package_txn(package_address, package_hash);
        }

        spec package_txn_prologue {
            aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
            include CheckPackageTxnAbortsIf{};
        }

        public fun package_txn_prologue_v2(account: &signer, txn_sender: address, package_address: address, package_hash: vector<u8>) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
            // Can only be invoked by genesis account
            CoreAddresses::assert_genesis_address(account);
            check_package_txn_v2(txn_sender, package_address, package_hash);
        }

        /// Package txn finished, and clean UpgradePlan
        public fun package_txn_epilogue(account: &signer, _txn_sender: address, package_address: address, success: bool) acquires TwoPhaseUpgradeV2, ModuleUpgradeStrategy {
            // Can only be invoked by genesis account
            CoreAddresses::assert_genesis_address(account);
            let strategy = get_module_upgrade_strategy(package_address);
            if(strategy == STRATEGY_TWO_PHASE){
                if (success) {
                    finish_upgrade_plan(package_address);
                };
            };
        }

        spec schema AbortsIfPackageTxnEpilogue {
            is_package: bool;
            package_address: address;
            success: bool;
            aborts_if is_package && get_module_upgrade_strategy(package_address) == STRATEGY_TWO_PHASE && success && !exists<TwoPhaseUpgrade>(package_address);
        }

        spec package_txn_epilogue {
            aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1
                && success && !exists<TwoPhaseUpgrade>(package_address);
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1
                && success && Option::is_some(global<TwoPhaseUpgrade>(package_address).plan)
                && !exists<Config::Config<Version::Version>>(global<TwoPhaseUpgrade>(package_address).version_cap.account_address);
        }

    }
}