// Copyright (c) The Starcoin Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag, CORE_CODE_ADDRESS};
use serde::{Deserialize, Serialize};

const MV_FLEXI_DAG_CONFIG_MODULE_NAME: &str = "FlexiDagConfig";
const MV_FLEXI_DAG_CONFIG_STRUCT_NAME: &str = "FlexiDagConfig";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct FlexiDagConfig {
    pub effective_height: u64,
}

impl OnChainConfig for FlexiDagConfig {
    const MODULE_IDENTIFIER: &'static str = MV_FLEXI_DAG_CONFIG_MODULE_NAME;
    const CONF_IDENTIFIER: &'static str = MV_FLEXI_DAG_CONFIG_STRUCT_NAME;
}

impl FlexiDagConfig {
    pub fn type_tag() -> TypeTag {
        TypeTag::Struct(Box::new(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new(MV_FLEXI_DAG_CONFIG_MODULE_NAME).unwrap(),
            name: Identifier::new(MV_FLEXI_DAG_CONFIG_STRUCT_NAME).unwrap(),
            type_args: vec![],
        }))
    }
}
