// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    gas_schedule::{CostTable, GasConstants},
    on_chain_config::OnChainConfig,
};
use anyhow::{format_err, Result};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag, CORE_CODE_ADDRESS};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

pub const SCRIPT_HASH_LENGTH: usize = HashValue::LENGTH;
const VM_CONFIG_MODULE_NAME: &str = "VMConfig";
pub static VM_CONFIG_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(VM_CONFIG_MODULE_NAME).unwrap());
pub static INSTRUCTION_SCHEDULE_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("instruction_schedule").unwrap());
pub static NATIVE_SCHEDULE_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("native_schedule").unwrap());
pub static GAS_CONSTANTS_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("gas_constants").unwrap());

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1.  !script_allowed && !module_publishing_allowed No module publishing, only script function are allowed.
/// 2.  script_allowed && !module_publishing_allowed No module publishing, custom scripts are allowed.
/// 3.  script_allowed && module_publishing_allowed Both module publishing and custom scripts are allowed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TransactionPublishOption {
    // Anyone can use custom script if this flag is set to true.
    script_allowed: bool,
    // Anyone can publish new module if this flag is set to true.
    module_publishing_allowed: bool,
}

impl TransactionPublishOption {
    pub fn locked() -> Self {
        Self {
            script_allowed: false,
            module_publishing_allowed: false,
        }
    }

    pub fn custom_scripts() -> Self {
        Self {
            script_allowed: true,
            module_publishing_allowed: false,
        }
    }

    pub fn open() -> Self {
        Self {
            script_allowed: true,
            module_publishing_allowed: true,
        }
    }

    pub fn is_module_publishing_allowed(&self) -> bool {
        self.module_publishing_allowed
    }

    pub fn is_script_allowed(&self) -> bool {
        self.script_allowed
    }
}

impl OnChainConfig for TransactionPublishOption {
    const MODULE_IDENTIFIER: &'static str = "TransactionPublishOption";
    const CONF_IDENTIFIER: &'static str = "TransactionPublishOption";

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let vm_publishing_option =
            bcs_ext::from_bytes::<TransactionPublishOption>(bytes).map_err(|e| {
                format_err!(
                    "Failed first round of deserialization for TransactionPublishOption: {}",
                    e
                )
            })?;
        Ok(vm_publishing_option)
    }
}

/// Defines all the on chain configuration data needed by VM.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[allow(clippy::upper_case_acronyms)]
pub struct VMConfig {
    pub gas_schedule: CostTable,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct CostTableInner {
    pub instruction_table: Vec<u8>,
    pub native_table: Vec<u8>,
    pub gas_constants: GasConstants,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[allow(clippy::upper_case_acronyms)]
struct VMConfigInner {
    pub gas_schedule: CostTableInner,
}

impl CostTableInner {
    pub fn as_cost_table(&self) -> Result<CostTable> {
        let instruction_table = bcs_ext::from_bytes(&self.instruction_table)?;
        let native_table = bcs_ext::from_bytes(&self.native_table)?;
        Ok(CostTable {
            instruction_table,
            native_table,
            gas_constants: self.gas_constants.clone(),
        })
    }
}

impl OnChainConfig for VMConfig {
    const MODULE_IDENTIFIER: &'static str = VM_CONFIG_MODULE_NAME;
    const CONF_IDENTIFIER: &'static str = VM_CONFIG_MODULE_NAME;

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let raw_vm_config = bcs_ext::from_bytes::<VMConfigInner>(bytes).map_err(|e| {
            format_err!(
                "Failed first round of deserialization for VMConfigInner: {}",
                e
            )
        })?;
        let gas_schedule = raw_vm_config.gas_schedule.as_cost_table()?;
        Ok(VMConfig { gas_schedule })
    }
}

pub fn vm_config_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: VM_CONFIG_IDENTIFIER.clone(),
        name: VM_CONFIG_IDENTIFIER.clone(),
        type_params: vec![],
    })
}
