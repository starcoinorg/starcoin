// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// ref aptos-move/aptos-gas-algebra/src/algebra.rs

pub use move_core_types::gas_algebra::*;

/// Unit of (external) gas.
pub enum GasUnit {}

/// Unit of the Starcoin network's native coin.
pub enum STC {}

/// Unit of gas currency. 1 NanoSTC = 10^-9 Starcoin coins.
pub enum NanoSTC {}

pub type Gas = GasQuantity<GasUnit>;

pub type GasScalingFactor = GasQuantity<UnitDiv<InternalGasUnit, GasUnit>>;

pub type Fee = GasQuantity<NanoSTC>;

pub type FeePerGasUnit = GasQuantity<UnitDiv<NanoSTC, GasUnit>>;

/// Unit of storage slot
pub enum Slot {}

pub type NumSlots = GasQuantity<Slot>;

pub type FeePerSlot = GasQuantity<UnitDiv<NanoSTC, Slot>>;

pub type FeePerByte = GasQuantity<UnitDiv<NanoSTC, Byte>>;

/***************************************************************************************************
 * Unit Conversion
 *
 **************************************************************************************************/
impl ToUnit<NanoSTC> for STC {
    const MULTIPLIER: u64 = 1_0000_0000;
}
