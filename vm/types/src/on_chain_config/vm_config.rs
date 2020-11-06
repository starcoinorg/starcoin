// Copyright (c) The Libra Core Contributors
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
static VM_CONFIG_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(VM_CONFIG_MODULE_NAME).unwrap());

/// Defines and holds the publishing policies for the VM. There are three possible configurations:
/// 1. No module publishing, only whitelisted scripts are allowed.
/// 2. No module publishing, custom scripts are allowed.
/// 3. Both module publishing and custom scripts are allowed.
/// We represent these as an enum instead of a struct since whitelisting and module/script
/// publishing are mutually exclusive options.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum VMPublishingOption {
    /// Only allow scripts on a whitelist to be run
    Locked(Vec<[u8; SCRIPT_HASH_LENGTH]>),
    /// Allow custom scripts, but _not_ custom module publishing
    CustomScripts,
    /// Allow both custom scripts and custom module publishing
    Open,
}

impl VMPublishingOption {
    pub fn is_open(&self) -> bool {
        match self {
            VMPublishingOption::Open => true,
            _ => false,
        }
    }

    pub fn is_allowed_script(&self, program: &[u8]) -> bool {
        match self {
            VMPublishingOption::Open | VMPublishingOption::CustomScripts => true,
            VMPublishingOption::Locked(whitelist) => {
                let hash_value = HashValue::sha3_256_of(program);
                whitelist.contains(hash_value.as_ref())
            }
        }
    }

    pub fn allowed_script(&self) -> Vec<[u8; SCRIPT_HASH_LENGTH]> {
        match self {
            VMPublishingOption::Open | VMPublishingOption::CustomScripts => Vec::new(),
            VMPublishingOption::Locked(whitelist) => whitelist.clone(),
        }
    }
}

/// Defines all the on chain configuration data needed by VM.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
struct VMConfigInner {
    pub gas_schedule: CostTableInner,
}

impl CostTableInner {
    pub fn as_cost_table(&self) -> Result<CostTable> {
        let instruction_table = scs::from_bytes(&self.instruction_table)?;
        let native_table = scs::from_bytes(&self.native_table)?;
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
        let raw_vm_config = scs::from_bytes::<VMConfigInner>(&bytes).map_err(|e| {
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
