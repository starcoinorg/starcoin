// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::access_path::AccessPath as LibraAccessPath;
use types::{
    access_path::AccessPath,
};

pub struct AccessPathHelper {}
impl AccessPathHelper {
    pub fn to_Starcoin_AccessPath(access_path: &LibraAccessPath) -> AccessPath {
        access_path.clone().into()
    }
}
