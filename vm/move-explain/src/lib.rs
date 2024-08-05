// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    errmap::{ErrorDescription, ErrorMapping},
};

/// Given the module ID and the abort code raised from that module, returns the human-readable
/// explanation of that abort if possible.
pub fn get_explanation(module_name: &str, abort_code: u64) -> Option<ErrorDescription> {
    let error_descriptions: ErrorMapping =
        bcs_ext::from_bytes(stdlib::ERROR_DESCRIPTIONS).expect("Decode err map failed");
    error_descriptions.get_explanation(module_name, abort_code)
}
