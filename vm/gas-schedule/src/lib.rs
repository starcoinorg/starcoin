// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// ref aptos-move/aptos-gas-schedule/src/lib.rs

mod gas_schedule;
mod traits;
mod ver;

pub use gas_schedule::*;
pub use move_vm_test_utils::gas_schedule::GasCost;
use starcoin_gas_algebra::{Arg, GasQuantity, UnitDiv};
pub use traits::{FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule};
pub use ver::LATEST_GAS_FEATURE_VERSION;

/// Unit of abstract value size -- a conceptual measurement of the memory space a Move value occupies.
pub enum AbstractValueUnit {}

pub type AbstractValueSize = GasQuantity<AbstractValueUnit>;

pub type AbstractValueSizePerArg = GasQuantity<UnitDiv<AbstractValueUnit, Arg>>;
