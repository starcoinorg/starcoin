// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag, CORE_CODE_ADDRESS};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

const VERSION_CONFIG_MODULE_NAME: &str = "Version";
pub static VERSION_CONFIG_IDENTIFIER: Lazy<Identifier> =
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

pub fn version_config_type_tag() -> TypeTag {
    TypeTag::Struct(StructTag {
        address: CORE_CODE_ADDRESS,
        module: VERSION_CONFIG_IDENTIFIER.clone(),
        name: VERSION_CONFIG_IDENTIFIER.clone(),
        type_params: vec![],
    })
}
