// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Breakdown of fee charge and refund for a transaction.
/// The structure is:
///
/// - Net charge or refund (not in the statement)
///    - total charge: total_charge_gas_units, matches `gas_used` in the on-chain `TransactionInfo`.
///      This is the sum of the sub-items below.
///
/// (keep this doc in sync with the `struct FeeStatement` in Move.)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FeeStatement {
    /// Total gas charge.
    total_charge_gas_units: u64,
}

impl FeeStatement {
    pub fn zero() -> Self {
        Self {
            total_charge_gas_units: 0,
        }
    }

    pub fn new(
        total_charge_gas_units: u64,
    ) -> Self {
        Self {
            total_charge_gas_units,
        }
    }
    pub fn gas_used(&self) -> u64 {
        self.total_charge_gas_units
    }

    pub fn add_fee_statement(&mut self, other: &FeeStatement) {
        self.total_charge_gas_units += other.total_charge_gas_units;
    }
}

