// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// ref aptos-move/aptos-gas-schedule/src/lib.rs

mod gas_schedule;
mod traits;
mod ver;

pub use gas_schedule::*;
pub use move_vm_test_utils::gas_schedule::GasCost;
use starcoin_gas_algebra::{AbstractValueSize, AbstractValueSizePerArg};
pub use traits::{FromOnChainGasSchedule, InitialGasSchedule, ToOnChainGasSchedule};
pub use ver::{gas_feature_versions, LATEST_GAS_FEATURE_VERSION};
