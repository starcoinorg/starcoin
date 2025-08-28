// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod instr;
mod macros;
mod misc;
mod move_stdlib;
mod starcoin_framework;

mod table;

mod nursery;
mod transaction;

use crate::{FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule};
pub use instr::InstructionGasParameters;
pub use misc::{AbstractValueSizeGasParameters, MiscGasParameters};
pub use move_stdlib::MoveStdlibGasParameters;
pub use nursery::NurseryGasParameters;
use once_cell::sync::Lazy;
pub use starcoin_framework::StarcoinFrameworkGasParameters;
use std::collections::BTreeMap;
pub use table::TableGasParameters;
pub use transaction::TransactionGasParameters;

pub mod gas_params {
    use super::*;
    pub use instr::gas_params as instr;
    pub use misc::gas_params as misc;
    pub use transaction::gas_params as txn;

    pub mod natives {
        use super::*;
        pub use move_stdlib::gas_params as move_stdlib;
        pub use nursery::gas_params as nursery;
        pub use starcoin_framework::gas_params as starcoin_framework;
        pub use table::gas_params as table;
    }
}

/// Gas parameters for everything that is needed to run the Starcoin blockchain, including
/// instructions, transactions and native functions from various packages.
#[derive(Debug, Clone)]
pub struct StarcoinGasParameters {
    pub vm: VMGasParameters,
    pub natives: NativeGasParameters,
}

impl FromOnChainGasSchedule for StarcoinGasParameters {
    fn from_on_chain_gas_schedule(
        gas_schedule: &BTreeMap<String, u64>,
        feature_version: u64,
    ) -> Result<Self, String> {
        Ok(Self {
            vm: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule, feature_version)?,
            natives: FromOnChainGasSchedule::from_on_chain_gas_schedule(
                gas_schedule,
                feature_version,
            )?,
        })
    }
}

impl ToOnChainGasSchedule for StarcoinGasParameters {
    fn to_on_chain_gas_schedule(&self, feature_version: u64) -> Vec<(String, u64)> {
        let mut entries = self.vm.to_on_chain_gas_schedule(feature_version);
        entries.extend(self.natives.to_on_chain_gas_schedule(feature_version));
        entries
    }
}

impl StarcoinGasParameters {
    // Only used for genesis and for tests where we need a cost table and
    // don't have a genesis storage state.
    pub fn zeros() -> Self {
        Self {
            vm: VMGasParameters::zeros(),
            natives: NativeGasParameters::zeros(),
        }
    }
}

impl InitialGasSchedule for StarcoinGasParameters {
    fn initial() -> Self {
        Self {
            vm: InitialGasSchedule::initial(),
            natives: InitialGasSchedule::initial(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VMGasParameters {
    pub misc: MiscGasParameters,
    pub instr: InstructionGasParameters,
    pub txn: TransactionGasParameters,
}

impl FromOnChainGasSchedule for VMGasParameters {
    fn from_on_chain_gas_schedule(
        gas_schedule: &BTreeMap<String, u64>,
        feature_version: u64,
    ) -> Result<Self, String> {
        Ok(Self {
            misc: FromOnChainGasSchedule::from_on_chain_gas_schedule(
                gas_schedule,
                feature_version,
            )?,
            instr: FromOnChainGasSchedule::from_on_chain_gas_schedule(
                gas_schedule,
                feature_version,
            )?,
            txn: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule, feature_version)?,
        })
    }
}

impl ToOnChainGasSchedule for VMGasParameters {
    fn to_on_chain_gas_schedule(&self, feature_version: u64) -> Vec<(String, u64)> {
        let mut entries = self.instr.to_on_chain_gas_schedule(feature_version);
        entries.extend(self.txn.to_on_chain_gas_schedule(feature_version));
        entries.extend(self.misc.to_on_chain_gas_schedule(feature_version));
        entries
    }
}

impl VMGasParameters {
    pub fn zeros() -> Self {
        Self {
            instr: InstructionGasParameters::zeros(),
            txn: TransactionGasParameters::zeros(),
            misc: MiscGasParameters::zeros(),
        }
    }
}

impl InitialGasSchedule for VMGasParameters {
    fn initial() -> Self {
        Self {
            instr: InitialGasSchedule::initial(),
            txn: InitialGasSchedule::initial(),
            misc: InitialGasSchedule::initial(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NativeGasParameters {
    pub move_stdlib: MoveStdlibGasParameters,
    pub table: TableGasParameters,
    pub nursery: NurseryGasParameters,
    pub starcoin_framework: StarcoinFrameworkGasParameters,
}

impl FromOnChainGasSchedule for NativeGasParameters {
    fn from_on_chain_gas_schedule(
        gas_schedule: &BTreeMap<String, u64>,
        feature_version: u64,
    ) -> Result<Self, String> {
        Ok(Self {
            move_stdlib: FromOnChainGasSchedule::from_on_chain_gas_schedule(
                gas_schedule,
                feature_version,
            )?,
            nursery: FromOnChainGasSchedule::from_on_chain_gas_schedule(
                gas_schedule,
                feature_version,
            )?,
            table: FromOnChainGasSchedule::from_on_chain_gas_schedule(
                gas_schedule,
                feature_version,
            )?,
            starcoin_framework: FromOnChainGasSchedule::from_on_chain_gas_schedule(
                gas_schedule,
                feature_version,
            )?,
        })
    }
}

impl ToOnChainGasSchedule for NativeGasParameters {
    fn to_on_chain_gas_schedule(&self, feature_version: u64) -> Vec<(String, u64)> {
        let mut entries = self.move_stdlib.to_on_chain_gas_schedule(feature_version);
        entries.extend(self.nursery.to_on_chain_gas_schedule(feature_version));
        entries.extend(self.table.to_on_chain_gas_schedule(feature_version));
        entries.extend(
            self.starcoin_framework
                .to_on_chain_gas_schedule(feature_version),
        );
        entries
    }
}

impl NativeGasParameters {
    pub fn zeros() -> Self {
        Self {
            move_stdlib: MoveStdlibGasParameters::zeros(),
            nursery: NurseryGasParameters::zeros(),
            table: TableGasParameters::zeros(),
            starcoin_framework: StarcoinFrameworkGasParameters::zeros(),
        }
    }
}

impl InitialGasSchedule for NativeGasParameters {
    fn initial() -> Self {
        Self {
            move_stdlib: InitialGasSchedule::initial(),
            nursery: InitialGasSchedule::initial(),
            table: InitialGasSchedule::initial(),
            starcoin_framework: StarcoinFrameworkGasParameters::initial(),
        }
    }
}

pub static G_LATEST_GAS_PARAMS: Lazy<StarcoinGasParameters> =
    Lazy::new(StarcoinGasParameters::initial);
