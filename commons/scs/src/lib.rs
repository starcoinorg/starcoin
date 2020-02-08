// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # Starcoin Canonical Serialization (SCS)
//!
//! SCS defines a deterministic means for translating a message or data structure into bytes
//! irrespective of platform, architecture, or programming language.

// Just a wrap to Libra Canonical Serialization (LCS) currently.
pub use lcs::{from_bytes, to_bytes, Error, Result, MAX_SEQUENCE_LENGTH};
