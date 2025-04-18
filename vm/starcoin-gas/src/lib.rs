// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate is the core of the gas metering system of the Starcoin blockchain.
//!
//! More specifically, it
//!   - Is home to the gas meter implementation
//!   - Defines the gas parameters and formulae for instructions
//!   - Defines the gas parameters for transactions
//!   - Sets the initial values for all gas parameters, including the instruction, transaction
//!     move-stdlib and starcoin-framework ones.
//!   - Defines a bi-directional mapping between the (Rust) gas parameter structs and their
//!     corresponding representation on-chain.
//!
//! The reason why we need two different representations is that they serve different purposes:
//!   - The Rust structs are used for quick (static) lookups by the gas meter and native functions
//!     when calculating gas costs.
//!   - The on-chain gas schedule needs to be extensible and unordered so we can upgrate it easily
//!     in the future.

mod gas_meter;

pub use gas_meter::{NativeGasParameters, StarcoinGasMeter, StarcoinGasParameters};
pub use move_core_types::gas_algebra::{
    Arg, Byte, GasQuantity, InternalGas, InternalGasPerArg, InternalGasPerByte, InternalGasUnit,
    NumArgs, NumBytes, UnitDiv,
};
pub use starcoin_gas_algebra_ext::InstructionGasParameters;
