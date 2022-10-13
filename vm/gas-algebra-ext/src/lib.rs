// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::gas_algebra::{Arg, GasQuantity, UnitDiv};
pub use move_vm_test_utils::gas_schedule::GasCost;
use serde::{Deserialize, Serialize};

#[macro_use]
pub mod natives;

#[macro_use]
pub mod params;

mod algebra;
mod gas_meter;
mod starcoin_framework;
//pub mod gen;
mod instr;
mod move_stdlib;
mod nursery;
mod table;
mod transaction;

pub use algebra::{FeePerGasUnit, Gas};
pub use gas_meter::{FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule};
pub use instr::InstructionGasParameters;
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
