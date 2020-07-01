address 0x1 {
    module UpgradeManager {
        use 0x1::Option::{Self,Option};
        use 0x1::Block;
        use 0x1::Signer;

        struct UpgradePlan {
            package_hash: vector<u8>,
            active_after_height: u64,
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

        // Update account's ModuleUpgradeStrategy
        public fun update_module_upgrade_strategy(account: &signer, strategy: u8) acquires ModuleUpgradeStrategy, TwoPhaseUpgrade, UpgradePlanCapability{
            assert(strategy == STRATEGY_ARBITRARY() || strategy == STRATEGY_TWO_PHASE() || strategy == STRATEGY_NEW_MODULE() || strategy == STRATEGY_FREEZE(), 1004);
            let account_address = Signer::address_of(account);
            let previous_strategy = get_module_upgrade_strategy(account_address);
            assert(strategy > previous_strategy, 1005);
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

        public fun destroy_upgrade_plan_cap(cap: UpgradePlanCapability){
            let UpgradePlanCapability{account_address:_} = cap;
        }

        public fun extract_submit_upgrade_plan_cap(account: &signer): UpgradePlanCapability acquires ModuleUpgradeStrategy, UpgradePlanCapability{
            let account_address = Signer::address_of(account);
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE(), 1006);
            move_from<UpgradePlanCapability>(account_address)
        }

        public fun submit_upgrade_plan(account: &signer, package_hash: vector<u8>, active_after_height: u64) acquires TwoPhaseUpgrade,UpgradePlanCapability,ModuleUpgradeStrategy{
            let account_address = Signer::address_of(account);
            let cap = borrow_global<UpgradePlanCapability>(account_address);
            submit_upgrade_plan_with_cap(cap, package_hash, active_after_height);
        }

        public fun submit_upgrade_plan_with_cap(cap: &UpgradePlanCapability, package_hash: vector<u8>, active_after_height: u64) acquires TwoPhaseUpgrade,ModuleUpgradeStrategy{
            assert(active_after_height >= Block::get_current_block_height(), 1005);
            let account_address = cap.account_address;
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE(), 1006);
            let tpu = borrow_global_mut<TwoPhaseUpgrade>(account_address);
            assert(Option::is_none(&tpu.plan), 1007);
            tpu.plan = Option::some(UpgradePlan{ package_hash, active_after_height});
        }

        public fun cancel_upgrade_plan(account: &signer) acquires TwoPhaseUpgrade,UpgradePlanCapability,ModuleUpgradeStrategy{
            let account_address = Signer::address_of(account);
            let cap = borrow_global<UpgradePlanCapability>(account_address);
            cancel_upgrade_plan_with_cap(cap);
        }

        public fun cancel_upgrade_plan_with_cap(cap: &UpgradePlanCapability) acquires TwoPhaseUpgrade,ModuleUpgradeStrategy{
            let account_address = cap.account_address;
            assert(get_module_upgrade_strategy(account_address) == STRATEGY_TWO_PHASE(), 1006);
            let tpu = borrow_global_mut<TwoPhaseUpgrade>(account_address);
            assert(Option::is_some(&tpu.plan), 1007);
            tpu.plan = Option::none<UpgradePlan>();
        }

        // Get Module maintainer for addr
        public fun get_module_maintainer(addr: address): address acquires ModuleMaintainer{
            if (exists<ModuleMaintainer>(addr)) {
                borrow_global<ModuleMaintainer>(addr).account_address
            }else{
                addr
            }
        }

        public fun get_module_upgrade_strategy(module_address: address): u8 acquires ModuleUpgradeStrategy {
            if (exists<ModuleUpgradeStrategy>(module_address)) {
                borrow_global<ModuleUpgradeStrategy>(module_address).strategy
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

        public fun check_module_upgrade(sender: address, module_address: address, package_hash: vector<u8>) acquires ModuleMaintainer, TwoPhaseUpgrade, ModuleUpgradeStrategy{
            let module_maintainer = get_module_maintainer(module_address);
            //TODO define error code.
            assert(module_maintainer == sender, 1000);
            let strategy = get_module_upgrade_strategy(module_address);
            if (strategy == STRATEGY_ARBITRARY()){
                //do nothing
            }else if(strategy == STRATEGY_TWO_PHASE()){
                let plan_opt = get_upgrade_plan(module_address);
                assert(Option::is_some(&plan_opt), 1001);
                let plan = Option::borrow(&plan_opt);
                assert(*&plan.package_hash == package_hash, 1002);
                assert(plan.active_after_height <= Block::get_current_block_height(), 1003);
            }else if(strategy == STRATEGY_NEW_MODULE()){
                //do check at VM
            }else if(strategy == STRATEGY_FREEZE()){
                abort(1004)
            };
        }

        public fun module_upgrade_epilogue(_sender: address, module_address: address, success: bool) acquires TwoPhaseUpgrade, ModuleUpgradeStrategy {
            //TODO limit call account
            let strategy = get_module_upgrade_strategy(module_address);
            if(strategy == STRATEGY_TWO_PHASE()){
                if (success) {
                    let tpu = borrow_global_mut<TwoPhaseUpgrade>(module_address);
                    tpu.plan = Option::none<UpgradePlan>();
                    //TODO fire event.
                };
            };
        }

    }
}