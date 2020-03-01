// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::access_path::AccessPath as LibraAccessPath;
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
};

pub struct AccessPathHelper {}
impl AccessPathHelper {
    pub fn to_Starcoin_AccessPath(access_path: &LibraAccessPath) -> AccessPath {
        // ToDo: fix me
        AccessPath {
            address: AccountAddress::new([1u8; ADDRESS_LENGTH]),
            path: b"/foo/c".to_vec(),
        }

        //types::access_path::AccessPath::new(access_path.address, access_path.path )
    }
}
