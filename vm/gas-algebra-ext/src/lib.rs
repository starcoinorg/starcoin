// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// ref aptos-move/aptos-gas-algebra/src/lib.rs

pub use crate::{
    misc::MiscGasParameters, move_stdlib::MoveStdlibGasParameters, nursery::NurseryGasParameters,
    table::TableGasParameters,
};
use move_core_types::gas_algebra::Arg;
pub use move_vm_test_utils::gas_schedule::GasCost;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// some code need to refactor starcoin-gas-schedule

#[macro_use]
pub mod macros;

mod algebra;
mod starcoin_framework;
mod traits;
//pub mod gen;
mod abstract_algebra;
mod instr;
mod misc;
mod move_stdlib;
mod nursery;
mod table;
mod transaction;

pub use abstract_algebra::*;
pub use algebra::*;
pub use algebra::{FeePerGasUnit, Gas};
pub use instr::InstructionGasParameters;
pub use traits::{FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule};
pub use transaction::TransactionGasParameters;

/// Unit of abstract value size -- a conceptual measurement of the memory space a Move value occupies.
pub enum AbstractValueUnit {}

pub type AbstractValueSize = GasQuantity<AbstractValueUnit>;

pub type AbstractValueSizePerArg = GasQuantity<UnitDiv<AbstractValueUnit, Arg>>;

#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct GasConstants {
    /// The cost per-byte read from global storage.
    pub global_memory_per_byte_cost: u64,

    /// The cost per-byte written to storage.
    pub global_memory_per_byte_write_cost: u64,

    /// The flat minimum amount of gas required for any transaction.
    /// Charged at the start of execution.
    pub min_transaction_gas_units: u64,

    /// Any transaction over this size will be charged an additional amount per byte.
    pub large_transaction_cutoff: u64,

    /// The units of gas that to be charged per byte over the `large_transaction_cutoff` in addition to
    /// `min_transaction_gas_units` for transactions whose size exceeds `large_transaction_cutoff`.
    pub intrinsic_gas_per_byte: u64,

    /// ~5 microseconds should equal one unit of computational gas. We bound the maximum
    /// computational time of any given transaction at roughly 20 seconds. We want this number and
    /// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
    /// MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
    /// NB: The bound is set quite high since custom scripts aren't allowed except from predefined
    /// and vetted senders.
    pub maximum_number_of_gas_units: u64,

    /// The minimum gas price that a transaction can be submitted with.
    pub min_price_per_gas_unit: u64,

    /// The maximum gas unit price that a transaction can be submitted with.
    pub max_price_per_gas_unit: u64,

    pub max_transaction_size_in_bytes: u64,

    pub gas_unit_scaling_factor: u64,
    pub default_account_size: u64,
}

/// The cost tables, keyed by the serialized form of the bytecode instruction.  We use the
/// serialized form as opposed to the instruction enum itself as the key since this will be the
/// on-chain representation of bytecode instructions in the future.
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Deserialize)]
pub struct CostTable {
    pub instruction_table: Vec<GasCost>,
    pub native_table: Vec<GasCost>,
    pub gas_constants: GasConstants,
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
    pub nursery: NurseryGasParameters,
    pub table: TableGasParameters,
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
        })
    }
}

impl ToOnChainGasSchedule for NativeGasParameters {
    fn to_on_chain_gas_schedule(&self, feature_version: u64) -> Vec<(String, u64)> {
        let mut entries = self.move_stdlib.to_on_chain_gas_schedule(feature_version);
        entries.extend(self.nursery.to_on_chain_gas_schedule(feature_version));
        entries.extend(self.table.to_on_chain_gas_schedule(feature_version));
        entries
    }
}

impl NativeGasParameters {
    pub fn zeros() -> Self {
        Self {
            move_stdlib: MoveStdlibGasParameters::zeros(),
            nursery: NurseryGasParameters::zeros(),
            table: TableGasParameters::zeros(),
        }
    }
}

impl InitialGasSchedule for NativeGasParameters {
    fn initial() -> Self {
        Self {
            move_stdlib: InitialGasSchedule::initial(),
            nursery: InitialGasSchedule::initial(),
            table: InitialGasSchedule::initial(),
        }
    }
}
