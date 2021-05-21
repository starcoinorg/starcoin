// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::StdlibVersion;
use starcoin_crypto::HashValue;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::{core_code_address, genesis_address};
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_vm_types::transaction::ScriptFunction;

pub trait StdlibCompat {
    fn upgrade_module_type_tag(&self) -> TypeTag;

    fn propose_module_upgrade_function(
        &self,
        token_type: TypeTag,
        module_address: AccountAddress,
        package_hash: HashValue,
        exec_delay: u64,
        enforced: bool,
    ) -> ScriptFunction;
}

impl StdlibCompat for StdlibVersion {
    fn upgrade_module_type_tag(&self) -> TypeTag {
        let struct_name = if self > &StdlibVersion::Version(2) {
            "UpgradeModuleV2"
        } else {
            "UpgradeModule"
        };
        TypeTag::Struct(StructTag {
            address: genesis_address(),
            module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
            name: Identifier::new(struct_name).unwrap(),
            type_params: vec![],
        })
    }

    fn propose_module_upgrade_function(
        &self,
        token_type: TypeTag,
        module_address: AccountAddress,
        package_hash: HashValue,
        exec_delay: u64,
        enforced: bool,
    ) -> ScriptFunction {
        // propose_module_upgrade_v2 is available after v2 upgrade.
        // 'self' is the target stdlib version to be upgraded to.
        let (function_name, args) = if self > &StdlibVersion::Version(2) {
            (
                "propose_module_upgrade_v2",
                vec![
                    bcs_ext::to_bytes(&module_address).unwrap(),
                    bcs_ext::to_bytes(&package_hash.to_vec()).unwrap(),
                    bcs_ext::to_bytes(&self.version()).unwrap(),
                    bcs_ext::to_bytes(&exec_delay).unwrap(),
                    bcs_ext::to_bytes(&enforced).unwrap(),
                ],
            )
        } else {
            (
                "propose_module_upgrade",
                vec![
                    bcs_ext::to_bytes(&module_address).unwrap(),
                    bcs_ext::to_bytes(&package_hash.to_vec()).unwrap(),
                    bcs_ext::to_bytes(&self.version()).unwrap(),
                    bcs_ext::to_bytes(&exec_delay).unwrap(),
                ],
            )
        };

        ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("ModuleUpgradeScripts").unwrap(),
            ),
            Identifier::new(function_name).unwrap(),
            vec![token_type],
            args,
        )
    }
}
