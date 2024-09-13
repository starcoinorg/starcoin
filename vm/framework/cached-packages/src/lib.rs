// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::Lazy;
use starcoin_framework::ReleaseBundle;

pub mod starcoin_framework_sdk_builder;
pub mod starcoin_stdlib;
pub mod starcoin_token_objects_sdk_builder;
pub mod starcoin_token_sdk_builder;

#[cfg(unix)]
const HEAD_RELEASE_BUNDLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/head.mrb"));
#[cfg(windows)]
const HEAD_RELEASE_BUNDLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "\\head.mrb"));

static HEAD_RELEASE_BUNDLE: Lazy<ReleaseBundle> = Lazy::new(|| {
    bcs::from_bytes::<ReleaseBundle>(HEAD_RELEASE_BUNDLE_BYTES).expect("bcs succeeds")
});

/// Returns the release bundle for the current code.
pub fn head_release_bundle() -> &'static ReleaseBundle {
    &HEAD_RELEASE_BUNDLE
}
