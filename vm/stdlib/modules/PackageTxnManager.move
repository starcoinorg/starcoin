address 0x1 {
    module PackageTxnManager {
        use 0x1::Option::{Self,Option};
        use 0x1::Signer;
        use 0x1::CoreAddresses;
        use 0x1::Block;
        use 0x1::Errors;
        use 0x1::Version;
        use 0x1::Event;
        use 0x1::Config;

        spec module {
            pragma verify = true;
            pragma aborts_if_is_strict = true;
        }

        struct UpgradePlan {
            package_hash: vector<u8>,
            active_after_number: u64,
            version: u64,
        }

        // The holder of UpgradePlanCapability for account_address can submit UpgradePlan for account_address.
        resource struct UpgradePlanCapability{
            account_address: address,
        }

        const STRATEGY_ARBITRARY: u8 = 0;
        const STRATEGY_TWO_PHASE: u8 = 1;
        const STRATEGY_NEW_MODULE: u8 = 2;
        const STRATEGY_FREEZE: u8 = 3;

        public fun get_strategy_arbitrary(): u8 { STRATEGY_ARBITRARY }

        public fun get_strategy_two_phase(): u8 { STRATEGY_TWO_PHASE }

        public fun get_strategy_new_module(): u8 { STRATEGY_NEW_MODULE }

        public fun get_strategy_freeze(): u8 { STRATEGY_FREEZE }

        const EUPGRADE_PLAN_IS_NONE: u64 = 102;
        const EPACKAGE_HASH_INCORRECT: u64 = 103;
        const EACTIVE_TIME_INCORRECT: u64 = 104;
        const ESTRATEGY_FREEZED: u64 = 105;
        const ESTRATEGY_INCORRECT: u64 = 106;
        const ESTRATEGY_NOT_TWO_PHASE: u64 = 107;
        const EUPGRADE_PLAN_IS_NOT_NONE: u64 = 108;
        const EUNKNOWN_STRATEGY: u64 = 109;

        resource struct ModuleUpgradeStrategy {
            // 0 arbitrary
            // 1 two phase upgrade
            // 2 only new module
            // 3 freeze
            strategy: u8,
        }

        resource struct TwoPhaseUpgrade {
            plan: Option<UpgradePlan>,
            version_cap: Config::ModifyConfigCapability<Version::Version>,
            upgrade_event: Event::EventHandle<Self::UpgradeEvent>,
        }

        struct UpgradeEvent {
            package_address: address,
            package_hash: vector<u8>,
            version: u64,
        }

        // Update account's ModuleUpgradeStrategy
        public fun update_module_upgrade_strategy(account: &signer, strategy: u8) acquires ModuleUpgradeStrategy, TwoPhaseUpgrade, UpgradePlanCapability{
            assert(strategy == STRATEGY_ARBITRARY || strategy == STRATEGY_TWO_PHASE || strategy == STRATEGY_NEW_MODULE || strategy == STRATEGY_FREEZE, Errors::invalid_argument(EUNKNOWN_STRATEGY));
            let account_address = Signer::address_of(account);
            let previous_strategy = get_module_upgrade_strategy(account_address);
            assert(strategy > previous_strategy, Errors::invalid_argument(ESTRATEGY_INCORRECT));
            if (exists<ModuleUpgradeStrategy>(account_address)) {
                borrow_global_mut<ModuleUpgradeStrategy>(account_address).strategy = strategy;
            }else{
                move_to(account, ModuleUpgradeStrategy{ strategy: strategy});
            };
            if (strategy == STRATEGY_TWO_PHASE){
                move_to(account, UpgradePlanCapability{ account_address: account_address});
                move_to(account, TwoPhaseUpgrade{plan: Option::none<UpgradePlan>(), version_cap: Config::publish_new_config_with_capability<Version::Version>(account, Version::new_version(1)), upgrade_event: Event::new_event_handle<Self::UpgradeEvent>(account)});
            };
            //clean two phase upgrade resource
            if (previous_strategy == STRATEGY_TWO_PHASE){
                let tpu = move_from<TwoPhaseUpgrade>(account_address);
                let TwoPhaseUpgrade{plan:_, version_cap, upgrade_event} = tpu;
                Event::destroy_handle<Self::UpgradeEvent>(upgrade_event);
                Config::destory_modify_config_capability<Version::Version>(version_cap);
                // UpgradePlanCapability may be extracted
                if (exists<UpgradePlanCapability>(account_address)){
                    let cap = move_from<UpgradePlanCapability>(account_address);
                    destroy_upgrade_plan_cap(cap);
                };
            };
        }

        spec fun update_module_upgrade_strategy {
            aborts_if strategy != 0 && strategy != 1 && strategy != 2 && strategy != 3;
            aborts_if exists<ModuleUpgradeStrategy>(Signer::address_of(account)) && strategy <= global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy;
            aborts_if !exists<ModuleUpgradeStrategy>(Signer::address_of(account)) && strategy == 0;

            aborts_if strategy == 1 && exists<UpgradePlanCapability>(Signer::address_of(account));
            aborts_if strategy == 1 && exists<TwoPhaseUpgrade>(Signer::address_of(account));

            aborts_if exists<ModuleUpgradeStrategy>(Signer::address_of(account)) && global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy == 1
                    && !exists<TwoPhaseUpgrade>(Signer::address_of(account));
        }

        public fun account_address(cap: &UpgradePlanCapability): address {
            cap.account_address
        }

        public fun destroy_upgrade_plan_cap(cap: UpgradePlanCapability){
            let UpgradePlanCapability{account_address:_} = cap;
        }

        spec fun destroy_upgrade_plan_cap {
            aborts_if false;
        }

        public fun extract_submit_upgrade_plan_cap(account: &signer): UpgradePlanCapability acquires ModuleUpgradeStrategy, UpgradePlanCapability{
            let account_address = Signer::address_of(account);
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE, Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            move_from<UpgradePlanCapability>(account_address)
        }

        spec fun extract_submit_upgrade_plan_cap {
            aborts_if !exists<ModuleUpgradeStrategy>(Signer::address_of(account));
            aborts_if global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy != 1;
            aborts_if !exists<UpgradePlanCapability>(Signer::address_of(account));
        }

        public fun submit_upgrade_plan(account: &signer, package_hash: vector<u8>, version:u64, active_after_number: u64) acquires TwoPhaseUpgrade,UpgradePlanCapability,ModuleUpgradeStrategy{
            let account_address = Signer::address_of(account);
            let cap = borrow_global<UpgradePlanCapability>(account_address);
            submit_upgrade_plan_with_cap(cap, package_hash, version, active_after_number);
        }

        spec fun submit_upgrade_plan {
            aborts_if !exists<UpgradePlanCapability>(Signer::address_of(account));
            include SubmitUpgradePlanWithCapAbortsIf{account: global<UpgradePlanCapability>(Signer::address_of(account)).account_address};
            ensures Option::spec_is_some(global<TwoPhaseUpgrade>(global<UpgradePlanCapability>(Signer::address_of(account)).account_address).plan);
        }

        public fun submit_upgrade_plan_with_cap(cap: &UpgradePlanCapability, package_hash: vector<u8>, version: u64, active_after_number: u64) acquires TwoPhaseUpgrade,ModuleUpgradeStrategy{
            assert(active_after_number >= Block::get_current_block_number(), Errors::invalid_argument(EACTIVE_TIME_INCORRECT));
            let account_address = cap.account_address;
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE, Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            let tpu = borrow_global_mut<TwoPhaseUpgrade>(account_address);
            assert(Option::is_none(&tpu.plan), Errors::invalid_state(EUPGRADE_PLAN_IS_NOT_NONE));
            tpu.plan = Option::some(UpgradePlan{ package_hash, active_after_number, version});
        }

        spec fun submit_upgrade_plan_with_cap {
            include SubmitUpgradePlanWithCapAbortsIf{account: cap.account_address, active_after_number};
            ensures Option::spec_is_some(global<TwoPhaseUpgrade>(cap.account_address).plan);
        }

        spec schema SubmitUpgradePlanWithCapAbortsIf {
            account: address;
            active_after_number: u64;
            aborts_if !exists<Block::BlockMetadata>(CoreAddresses::GENESIS_ADDRESS());
            aborts_if active_after_number < global<Block::BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).number;
            aborts_if !exists<ModuleUpgradeStrategy>(account);
            aborts_if global<ModuleUpgradeStrategy>(account).strategy != 1;
            aborts_if !exists<TwoPhaseUpgrade>(account);
            aborts_if !Option::spec_is_none(global<TwoPhaseUpgrade>(account).plan);
        }

        public fun cancel_upgrade_plan(account: &signer) acquires TwoPhaseUpgrade,UpgradePlanCapability,ModuleUpgradeStrategy{
            let account_address = Signer::address_of(account);
            let cap = borrow_global<UpgradePlanCapability>(account_address);
            cancel_upgrade_plan_with_cap(cap);
        }

        spec fun cancel_upgrade_plan {
            aborts_if !exists<UpgradePlanCapability>(Signer::address_of(account));
            include CancelUpgradePlanWithCapAbortsIf{account: global<UpgradePlanCapability>(Signer::address_of(account)).account_address};
            ensures Option::spec_is_none(global<TwoPhaseUpgrade>(global<UpgradePlanCapability>(Signer::address_of(account)).account_address).plan);
        }

        public fun cancel_upgrade_plan_with_cap(cap: &UpgradePlanCapability) acquires TwoPhaseUpgrade,ModuleUpgradeStrategy{
            let account_address = cap.account_address;
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE, Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            let tpu = borrow_global_mut<TwoPhaseUpgrade>(account_address);
            assert(Option::is_some(&tpu.plan), Errors::invalid_state(EUPGRADE_PLAN_IS_NONE));
            tpu.plan = Option::none<UpgradePlan>();
        }

        spec fun cancel_upgrade_plan_with_cap {
            include CancelUpgradePlanWithCapAbortsIf{account: cap.account_address};
            ensures Option::spec_is_none(global<TwoPhaseUpgrade>(cap.account_address).plan);
        }

        spec schema CancelUpgradePlanWithCapAbortsIf {
            account: address;
            aborts_if !exists<ModuleUpgradeStrategy>(account);
            aborts_if global<ModuleUpgradeStrategy>(account).strategy != 1;
            aborts_if !exists<TwoPhaseUpgrade>(account);
            aborts_if !Option::spec_is_some(global<TwoPhaseUpgrade>(account).plan);
        }

        public fun get_module_upgrade_strategy(module_address: address): u8 acquires ModuleUpgradeStrategy {
            if (exists<ModuleUpgradeStrategy>(module_address)) {
                borrow_global<ModuleUpgradeStrategy>(module_address).strategy
            }else{
                0
            }
        }

        spec fun get_module_upgrade_strategy {
            aborts_if false;
        }

        spec define spec_get_module_upgrade_strategy(module_address: address): u8 {
            if (exists<ModuleUpgradeStrategy>(module_address)) {
                global<ModuleUpgradeStrategy>(module_address).strategy
            }else{
                0
            }
        }

        public fun get_upgrade_plan(module_address: address): Option<UpgradePlan> acquires TwoPhaseUpgrade {
            if (exists<TwoPhaseUpgrade>(module_address)) {
                *&borrow_global<TwoPhaseUpgrade>(module_address).plan
            }else{
                Option::none<UpgradePlan>()
            }
        }

        spec fun get_upgrade_plan {
            aborts_if false;
        }

        spec define spec_get_upgrade_plan(module_address: address): Option<UpgradePlan> {
            if (exists<TwoPhaseUpgrade>(module_address)) {
                global<TwoPhaseUpgrade>(module_address).plan
            }else{
                Option::spec_none<UpgradePlan>()
            }
        }

        public fun check_package_txn(package_address: address, package_hash: vector<u8>) acquires TwoPhaseUpgrade, ModuleUpgradeStrategy{
            let strategy = get_module_upgrade_strategy(package_address);
            if (strategy == STRATEGY_ARBITRARY){
                //do nothing
            }else if(strategy == STRATEGY_TWO_PHASE){
                let plan_opt = get_upgrade_plan(package_address);
                assert(Option::is_some(&plan_opt), Errors::invalid_argument(EUPGRADE_PLAN_IS_NONE));
                let plan = Option::borrow(&plan_opt);
                assert(*&plan.package_hash == package_hash, Errors::invalid_argument(EPACKAGE_HASH_INCORRECT));
                assert(plan.active_after_number <= Block::get_current_block_number(), Errors::invalid_argument(EACTIVE_TIME_INCORRECT));
            }else if(strategy == STRATEGY_NEW_MODULE){
                //do check at VM runtime.
            }else if(strategy == STRATEGY_FREEZE){
                abort(ESTRATEGY_FREEZED)
            };
        }

        spec fun check_package_txn {
            include CheckPackageTxnAbortsIf;
        }

        spec schema CheckPackageTxnAbortsIf {
            package_address: address;
            package_hash: vector<u8>;
            aborts_if spec_get_module_upgrade_strategy(package_address) == 3;
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1 && Option::spec_is_none(spec_get_upgrade_plan(package_address));
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1 && Option::spec_get(spec_get_upgrade_plan(package_address)).package_hash != package_hash;
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1 && !exists<Block::BlockMetadata>(CoreAddresses::GENESIS_ADDRESS());
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1 && Option::spec_get(spec_get_upgrade_plan(package_address)).active_after_number > global<Block::BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).number;
        }

        spec schema CheckPackageTxnAbortsIfWithType {
            is_package: bool;
            sender: address;
            package_address: address;
            package_hash: vector<u8>;
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 3;
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && Option::spec_is_none(spec_get_upgrade_plan(package_address));
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && Option::spec_get(spec_get_upgrade_plan(package_address)).package_hash != package_hash;
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && !exists<Block::BlockMetadata>(CoreAddresses::GENESIS_ADDRESS());
            aborts_if is_package && spec_get_module_upgrade_strategy(package_address) == 1 && Option::spec_get(spec_get_upgrade_plan(package_address)).active_after_number > global<Block::BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).number;
        }

        fun finish_upgrade_plan(package_address: address) acquires TwoPhaseUpgrade {
            let tpu = borrow_global_mut<TwoPhaseUpgrade>(package_address);
            assert(Option::is_some(&tpu.plan), Errors::invalid_state(EUPGRADE_PLAN_IS_NONE));
            let plan = Option::borrow(&tpu.plan);
            Config::set_with_capability<Version::Version>(&mut tpu.version_cap, Version::new_version(plan.version));
            Event::emit_event<Self::UpgradeEvent>(&mut tpu.upgrade_event, UpgradeEvent {
                package_address: package_address,
                package_hash: *&plan.package_hash,
                version: plan.version});
            tpu.plan = Option::none<UpgradePlan>();
        }

        spec fun finish_upgrade_plan {
            aborts_if !exists<TwoPhaseUpgrade>(package_address);
        }

        public fun package_txn_prologue(account: &signer, package_address: address, package_hash: vector<u8>) acquires TwoPhaseUpgrade, ModuleUpgradeStrategy {
            // Can only be invoked by genesis account
            CoreAddresses::assert_genesis_address(account);
            check_package_txn(package_address, package_hash);
        }

        spec fun package_txn_prologue {
            aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
            include CheckPackageTxnAbortsIf{};
        }

        /// Package txn finished, and clean UpgradePlan
        public fun package_txn_epilogue(account: &signer, _txn_sender: address, package_address: address, success: bool) acquires TwoPhaseUpgrade, ModuleUpgradeStrategy {
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

        spec fun package_txn_epilogue {
            aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
            aborts_if spec_get_module_upgrade_strategy(package_address) == 1
                    && success && !exists<TwoPhaseUpgrade>(package_address);
    }

    }
}