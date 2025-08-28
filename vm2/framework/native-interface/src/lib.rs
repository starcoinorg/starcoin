// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// ref aptos-move/aptos-native-interface/src/lib.rs

//! This crate defines interfaces that enable the extension of the Starcoin VM with native functions.
//!
//! Native functions provide the ability to incorporate semantics that cannot be expressed in
//! normal Move programs.
//!
//! They are also commonly used to accelerate certain operations, such as cryptographic hashes,
//! by executing them in native code.

mod builder;
mod context;
mod errors;
mod native;

#[macro_use]
mod helpers;

#[doc(hidden)]
pub mod reexports;

pub use builder::SafeNativeBuilder;
pub use context::SafeNativeContext;
pub use errors::{SafeNativeError, SafeNativeResult};
pub use native::RawSafeNative;
