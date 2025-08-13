// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Re-export proptest_types from vm-types when available
// Note: This requires vm-types to be compiled with test or fuzzing feature
#[cfg(all(any(test, feature = "fuzzing"), feature = "fuzzing"))]
pub use starcoin_vm_types::proptest_types::*;
