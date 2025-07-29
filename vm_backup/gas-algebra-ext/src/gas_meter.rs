// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the official gas meter implementation, along with some top-level gas
//! parameters and traits to help manipulate them.

use std::collections::BTreeMap;

pub(crate) const EXECUTION_GAS_MULTIPLIER: u64 = 1;

/// A trait for converting from a map representation of the on-chain gas schedule.
pub trait FromOnChainGasSchedule: Sized {
    /// Constructs a value of this type from a map representation of the on-chain gas schedule.
    /// `None` should be returned when the gas schedule is missing some required entries.
    /// Unused entries should be safely ignored.
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self>;
}

/// A trait for converting to a list of entries of the on-chain gas schedule.
pub trait ToOnChainGasSchedule {
    /// Converts `self` into a list of entries of the on-chain gas schedule.
    /// Each entry is a key-value pair where the key is a string representing the name of the
    /// parameter, where the value is the gas parameter itself.
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)>;
}

/// A trait for defining an initial value to be used in the genesis.
pub trait InitialGasSchedule: Sized {
    /// Returns the initial value of this type, which is used in the genesis.
    fn initial() -> Self;
}
