address 0x1 {
    module PackageTxnManager {
        use 0x1::Option::{Self,Option};
        use 0x1::Signer;
        use 0x1::CoreAddresses;
        use 0x1::Block;
        use 0x1::Errors;

        spec module {
            pragma verify = true;
            pragma aborts_if_is_strict = true;
        }

        struct UpgradePlan {
            package_hash: vector<u8>,
            active_after_number: u64,
        }

        // The holder of UpgradePlanCapability for account_address can submit UpgradePlan for account_address.
        resource struct UpgradePlanCapability{
            account_address: address,
        }

        //TODO use constants when Move support constants
        public fun STRATEGY_ARBITRARY() : u8{0}
        public fun STRATEGY_TWO_PHASE() : u8{1}
        public fun STRATEGY_NEW_MODULE(): u8{2}
        public fun STRATEGY_FREEZE(): u8{3}

        const ESENDER_IS_NOT_MAINTAINER: u64 = 101;
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

        // The module maintainer of module in current code space. if this resource not exist, the maintainer is current account.
        resource struct ModuleMaintainer {
            account_address: address,
        }

        resource struct TwoPhaseUpgrade {
            plan: Option<UpgradePlan>,
        }

        // Grant account's module maintainer to `maintainer`
        public fun grant_maintainer(account: &signer, maintainer: address) acquires ModuleMaintainer{
           let account_address = Signer::address_of(account);
           if (exists<ModuleMaintainer>(account_address)) {
             borrow_global_mut<ModuleMaintainer>(account_address).account_address = maintainer;
           }else{
             move_to(account, ModuleMaintainer{ account_address: maintainer});
           };
        }

        spec fun grant_maintainer {
            aborts_if false;
        }

        // Update account's ModuleUpgradeStrategy
        public fun update_module_upgrade_strategy(account: &signer, strategy: u8) acquires ModuleUpgradeStrategy, TwoPhaseUpgrade, UpgradePlanCapability{
            assert(strategy == STRATEGY_ARBITRARY() || strategy == STRATEGY_TWO_PHASE() || strategy == STRATEGY_NEW_MODULE() || strategy == STRATEGY_FREEZE(), Errors::invalid_argument(EUNKNOWN_STRATEGY));
            let account_address = Signer::address_of(account);
            let previous_strategy = get_module_upgrade_strategy(account_address);
            assert(strategy > previous_strategy, Errors::invalid_argument(ESTRATEGY_INCORRECT));
            if (exists<ModuleUpgradeStrategy>(account_address)) {
                borrow_global_mut<ModuleUpgradeStrategy>(account_address).strategy = strategy;
            }else{
                move_to(account, ModuleUpgradeStrategy{ strategy: strategy});
            };
            if (strategy == STRATEGY_TWO_PHASE()){
                move_to(account, UpgradePlanCapability{ account_address: account_address});
                move_to(account, TwoPhaseUpgrade{plan: Option::none<UpgradePlan>()});
            };
            //clean two phase upgrade resource
            if (previous_strategy == STRATEGY_TWO_PHASE()){
                let tpu = move_from<TwoPhaseUpgrade>(account_address);
                let TwoPhaseUpgrade{plan:_} = tpu;
                // UpgradePlanCapability may be extracted
                if (exists<UpgradePlanCapability>(account_address)){
                    let cap = move_from<UpgradePlanCapability>(account_address);
                    destroy_upgrade_plan_cap(cap);
                };
            };
        }

        spec fun update_module_upgrade_strategy {
            pragma verify = false; //Todo: fix the spec below
            aborts_if strategy < 0 || strategy > 3;
            aborts_if strategy <= global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy;
            aborts_if global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy == 1
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
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE(), Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            move_from<UpgradePlanCapability>(account_address)
        }

        spec fun extract_submit_upgrade_plan_cap {
            aborts_if !exists<ModuleUpgradeStrategy>(Signer::address_of(account));
            aborts_if global<ModuleUpgradeStrategy>(Signer::address_of(account)).strategy != 1;
            aborts_if !exists<UpgradePlanCapability>(Signer::address_of(account));
        }

        public fun submit_upgrade_plan(account: &signer, package_hash: vector<u8>, active_after_number: u64) acquires TwoPhaseUpgrade,UpgradePlanCapability,ModuleUpgradeStrategy{
            let account_address = Signer::address_of(account);
            let cap = borrow_global<UpgradePlanCapability>(account_address);
            submit_upgrade_plan_with_cap(cap, package_hash, active_after_number);
        }

        spec fun submit_upgrade_plan {
            pragma verify = false;
        }

        public fun submit_upgrade_plan_with_cap(cap: &UpgradePlanCapability, package_hash: vector<u8>, active_after_number: u64) acquires TwoPhaseUpgrade,ModuleUpgradeStrategy{
            //FIXME
            //assert(active_after_number >= Block::get_current_block_number(), EACTIVE_TIME_INCORRECT());
            let account_address = cap.account_address;
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE(), Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            let tpu = borrow_global_mut<TwoPhaseUpgrade>(account_address);
            assert(Option::is_none(&tpu.plan), Errors::invalid_state(EUPGRADE_PLAN_IS_NOT_NONE));
            tpu.plan = Option::some(UpgradePlan{ package_hash, active_after_number});
        }

        spec fun submit_upgrade_plan_with_cap {
            pragma verify = false;
        }

        public fun cancel_upgrade_plan(account: &signer) acquires TwoPhaseUpgrade,UpgradePlanCapability,ModuleUpgradeStrategy{
            let account_address = Signer::address_of(account);
            let cap = borrow_global<UpgradePlanCapability>(account_address);
            cancel_upgrade_plan_with_cap(cap);
        }

        spec fun cancel_upgrade_plan {
            pragma verify = false;
        }

        public fun cancel_upgrade_plan_with_cap(cap: &UpgradePlanCapability) acquires TwoPhaseUpgrade,ModuleUpgradeStrategy{
            let account_address = cap.account_address;
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE(), Errors::invalid_argument(ESTRATEGY_NOT_TWO_PHASE));
            let tpu = borrow_global_mut<TwoPhaseUpgrade>(account_address);
            assert(Option::is_some(&tpu.plan), Errors::invalid_state(EUPGRADE_PLAN_IS_NONE));
            tpu.plan = Option::none<UpgradePlan>();
        }

        spec fun cancel_upgrade_plan_with_cap {
            pragma verify = false;
        }

        // Get Module maintainer for addr
        public fun get_module_maintainer(addr: address): address acquires ModuleMaintainer{
            if (exists<ModuleMaintainer>(addr)) {
                borrow_global<ModuleMaintainer>(addr).account_address
            }else{
                addr
            }
        }

        spec fun get_module_maintainer {
            pragma verify = false;
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

        public fun get_upgrade_plan(module_address: address): Option<UpgradePlan> acquires TwoPhaseUpgrade {
            if (exists<TwoPhaseUpgrade>(module_address)) {
                *&borrow_global<TwoPhaseUpgrade>(module_address).plan
            }else{
                Option::none<UpgradePlan>()
            }
        }

        spec fun get_upgrade_plan {
            pragma verify = false;
        }

        public fun check_package_txn(sender: address, package_address: address, package_hash: vector<u8>) acquires ModuleMaintainer, TwoPhaseUpgrade, ModuleUpgradeStrategy{
            let module_maintainer = get_module_maintainer(package_address);
            //TODO define error code.
            assert(module_maintainer == sender, Errors::requires_address(ESENDER_IS_NOT_MAINTAINER));
            let strategy = get_module_upgrade_strategy(package_address);
            if (strategy == STRATEGY_ARBITRARY()){
                //do nothing
            }else if(strategy == STRATEGY_TWO_PHASE()){
                let plan_opt = get_upgrade_plan(package_address);
                assert(Option::is_some(&plan_opt), Errors::invalid_argument(EUPGRADE_PLAN_IS_NONE));
                let plan = Option::borrow(&plan_opt);
                assert(*&plan.package_hash == package_hash, Errors::invalid_argument(EPACKAGE_HASH_INCORRECT));
                assert(plan.active_after_number <= Block::get_current_block_number(), Errors::invalid_argument(EACTIVE_TIME_INCORRECT));
            }else if(strategy == STRATEGY_NEW_MODULE()){
                //do check at VM runtime.
            }else if(strategy == STRATEGY_FREEZE()){
                abort(ESTRATEGY_FREEZED)
            };
        }

        spec fun check_package_txn {
            pragma verify = false;

        }

        fun finish_upgrade_plan(package_address: address) acquires TwoPhaseUpgrade {
            let tpu = borrow_global_mut<TwoPhaseUpgrade>(package_address);
            tpu.plan = Option::none<UpgradePlan>();
        }

        spec fun finish_upgrade_plan {
            aborts_if !exists<TwoPhaseUpgrade>(package_address);
        }

        public fun package_txn_prologue(account: &signer, txn_sender: address, package_address: address, package_hash: vector<u8>) acquires ModuleMaintainer, TwoPhaseUpgrade, ModuleUpgradeStrategy {
            // Can only be invoked by genesis account
            CoreAddresses::assert_genesis_address(account);
            check_package_txn(txn_sender, package_address, package_hash);
        }

        spec fun package_txn_prologue {
            pragma verify = false; //fix me
            aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        }

        /// Package txn finished, and clean UpgradePlan
        public fun package_txn_epilogue(account: &signer, _txn_sender: address, package_address: address, success: bool) acquires TwoPhaseUpgrade, ModuleUpgradeStrategy {
            // Can only be invoked by genesis account
            CoreAddresses::assert_genesis_address(account);
            let strategy = get_module_upgrade_strategy(package_address);
            if(strategy == STRATEGY_TWO_PHASE()){
                if (success) {
                    finish_upgrade_plan(package_address)
                    //TODO fire event.
                };
            };
        }

        spec fun package_txn_epilogue {
            pragma verify = false; //fix me
            aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
            aborts_if  global<ModuleUpgradeStrategy>(package_address).strategy == 1
                       && success && !exists<TwoPhaseUpgrade>(package_address);
        }

    }
}