// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::{OnChainConfig, StructTag, TypeTag, CORE_CODE_ADDRESS};
use move_core_types::ident_str;
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::{Deserialize, Serialize};

const MV_LANG_VERSION_MODULE_NAME: &str = "stc_language_version";
const MV_LANG_VERSION_STRUCT_NAME: &str = "LanguageVersion";

/// Defines the move language version.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct MoveLanguageVersion {
    pub major: u64,
}

impl OnChainConfig for MoveLanguageVersion {
    const MODULE_IDENTIFIER: &'static str = MV_LANG_VERSION_MODULE_NAME;
    const TYPE_IDENTIFIER: &'static str = MV_LANG_VERSION_STRUCT_NAME;
}
impl MoveStructType for MoveLanguageVersion {
    const MODULE_NAME: &'static IdentStr = ident_str!(MV_LANG_VERSION_MODULE_NAME);
    const STRUCT_NAME: &'static IdentStr = ident_str!(MV_LANG_VERSION_STRUCT_NAME);
}

impl MoveResource for MoveLanguageVersion {}

impl MoveLanguageVersion {
    pub fn type_tag() -> TypeTag {
        TypeTag::Struct(Box::new(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new(MV_LANG_VERSION_MODULE_NAME).unwrap(),
            name: Identifier::new(MV_LANG_VERSION_STRUCT_NAME).unwrap(),
            type_args: vec![],
        }))
    }
}
