// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

/// The number of directories between the project root and the root of this crate.
pub const X_DEPTH: usize = 2;

/// Returns the project root. TODO: switch uses to XCoreContext::project_root instead)
pub fn project_root() -> &'static Path {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(X_DEPTH)
        .unwrap()
}
