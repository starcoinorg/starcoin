// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::genesis_config::StdlibVersion;
use crate::on_chain_config::OnChainConfig;
use move_core_types::ident_str;
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::move_resource::MoveStructType;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const VERSION_CONFIG_MODULE_NAME: &str = "stc_version";
const VERSION_CONFIG_STRUCT_NAME: &str = "Version";

pub static G_VERSION_CONFIG_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(VERSION_CONFIG_MODULE_NAME).unwrap());

/// Defines the version of Starcoin software.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
}

impl Version {
    pub fn into_stdlib_version(self) -> StdlibVersion {
        if self.major == 0 {
            StdlibVersion::Latest
        } else {
            StdlibVersion::Version(self.major)
        }
    }
}

impl OnChainConfig for Version {
    const MODULE_IDENTIFIER: &'static str = VERSION_CONFIG_MODULE_NAME;
    const TYPE_IDENTIFIER: &'static str = VERSION_CONFIG_STRUCT_NAME;
}
impl MoveStructType for Version {
    const MODULE_NAME: &'static IdentStr = ident_str!(VERSION_CONFIG_MODULE_NAME);
    const STRUCT_NAME: &'static IdentStr = ident_str!(VERSION_CONFIG_STRUCT_NAME);
}
