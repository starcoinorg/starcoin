// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::ops::Deref;
use anyhow::Result;
use move_core_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::{access_path_for_module_upgrade_strategy, access_path_for_two_phase_upgrade_v2, genesis_address, ModuleUpgradeStrategy, TwoPhaseUpgradeV2Resource};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_view::{StateReaderExt, StateView};
use crate::data_cache::{RemoteStorageOwned, StateViewCache};
use crate::move_vm_ext::MoveResolverExt;
//
// pub struct VMExecuteStrategyParams<'a, S: StateView> {
//     data_cache: &'a StateViewCache<'a, S>,
// }
//
// impl<'a, S: StateView> VMExecuteStrategyParams<'a, S> {
//     pub fn new(data_cache: &'a StateViewCache<S>) -> VMExecuteStrategyParams<'a, S> {
//         Self {
//             data_cache
//         }
//     }
//
//     pub fn is_genesis(&self) -> bool {
//         self.data_cache.is_genesis()
//     }
//
//     pub fn only_new_module_strategy(
//         &self,
//         package_address: AccountAddress,
//     ) -> Result<bool> {
//         let strategy_access_path = access_path_for_module_upgrade_strategy(package_address);
//         if let Some(data) =
//             self.data_cache.get_state_value(&StateKey::AccessPath(strategy_access_path))?
//         {
//             Ok(bcs_ext::from_bytes::<ModuleUpgradeStrategy>(&data)?.only_new_module())
//         } else {
//             Ok(false)
//         }
//     }
//
//     pub fn is_enforced(
//         &self,
//         package_address: AccountAddress,
//     ) -> Result<bool> {
//         let chain_id = self.data_cache.get_chain_id()?;
//         let block_meta = self.data_cache.get_block_metadata()?;
//         // from mainnet after 8015088 and barnard after 8311392, we disable enforce upgrade
//         if package_address == genesis_address()
//             || (chain_id.is_main() && block_meta.number < 8015088)
//             || (chain_id.is_barnard() && block_meta.number < 8311392)
//         {
//             let two_phase_upgrade_v2_path = access_path_for_two_phase_upgrade_v2(package_address);
//             if let Some(data) =
//                 self.data_cache.get_state_value(&StateKey::AccessPath(two_phase_upgrade_v2_path))?
//             {
//                 let enforced = bcs_ext::from_bytes::<TwoPhaseUpgradeV2Resource>(&data)?.enforced();
//                 Ok(enforced)
//             } else {
//                 Ok(false)
//             }
//         } else {
//             Ok(false)
//         }
//     }
// }


pub struct VMExecuteStrategyParams {
}

impl VMExecuteStrategyParams {
    pub fn new() ->  VMExecuteStrategyParams {
        VMExecuteStrategyParams {

        }
    }

    pub fn is_genesis(&self) -> bool {
        //self.data_cache.is_genesis()
        return false
    }

    pub fn only_new_module_strategy(
        &self,
        _package_address: AccountAddress,
    ) -> Result<bool> {
        // let strategy_access_path = access_path_for_module_upgrade_strategy(package_address);
        // if let Some(data) =
        //     self.data_cache.get_state_value(&StateKey::AccessPath(strategy_access_path))?
        // {
        //     Ok(bcs_ext::from_bytes::<ModuleUpgradeStrategy>(&data)?.only_new_module())
        // } else {
        //     Ok(false)
        // }
        Ok(true)
    }

    pub fn is_enforced(
        &self,
        _package_address: AccountAddress,
    ) -> Result<bool> {
        Ok(true)
    }
}