// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::gas_algebra::{GasQuantity, InternalGasUnit, UnitDiv};

pub use gas_algebra_ext::{
    AbstractValueSize, AbstractValueSizePerArg, AbstractValueUnit, InternalGasPerAbstractValueUnit,
};

/// Unit of (external) gas.
pub enum GasUnit {}

/// Unit of gas currency. 1 NanoSTC = 10^-9 Starcoin coins.
pub enum NanoSTC {}

pub type Gas = GasQuantity<GasUnit>;

pub type GasScalingFactor = GasQuantity<UnitDiv<InternalGasUnit, GasUnit>>;

pub type Fee = GasQuantity<NanoSTC>;

pub type FeePerGasUnit = GasQuantity<UnitDiv<NanoSTC, GasUnit>>;
