// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use crate::on_chain_config::OnChainConfig;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag, CORE_CODE_ADDRESS};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const VERSION_CONFIG_MODULE_NAME: &str = "Version";
pub static G_VERSION_CONFIG_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(VERSION_CONFIG_MODULE_NAME).unwrap());

/// Defines the version of Starcoin software.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Version {
    pub major: u64,
}

impl OnChainConfig for Version {
    const MODULE_IDENTIFIER: &'static str = VERSION_CONFIG_MODULE_NAME;
    const CONF_IDENTIFIER: &'static str = VERSION_CONFIG_MODULE_NAME;
}
impl MoveResource for Version {
    const MODULE_NAME: &'static str = VERSION_CONFIG_MODULE_NAME;
    const STRUCT_NAME: &'static str = "Version";
}
pub fn version_config_type_tag() -> TypeTag {
    TypeTag::Struct(Box::new(StructTag {
        address: CORE_CODE_ADDRESS,
        module: G_VERSION_CONFIG_IDENTIFIER.clone(),
        name: G_VERSION_CONFIG_IDENTIFIER.clone(),
        type_params: vec![],
    }))
}
