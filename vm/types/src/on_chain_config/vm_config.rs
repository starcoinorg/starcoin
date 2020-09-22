// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    gas_schedule::{CostTable, GasConstants},
    on_chain_config::OnChainConfig,
};
use anyhow::{format_err, Result};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

pub const SCRIPT_HASH_LENGTH: usize = HashValue::LENGTH;

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
}

/// Defines all the on chain configuration data needed by VM.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct VMConfig {
    pub gas_schedule: CostTable,
    pub block_gas_limit: u64,
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
    pub block_gas_limit: u64,
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
    const IDENTIFIER: &'static str = "VMConfig";

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let raw_vm_config = scs::from_bytes::<VMConfigInner>(&bytes).map_err(|e| {
            format_err!(
                "Failed first round of deserialization for VMConfigInner: {}",
                e
            )
        })?;
        let gas_schedule = raw_vm_config.gas_schedule.as_cost_table()?;
        let block_gas_limit = raw_vm_config.block_gas_limit;
        Ok(VMConfig {
            gas_schedule,
            block_gas_limit,
        })
    }
}
