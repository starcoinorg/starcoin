// Copyright (c) The Starcoin Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag, CORE_CODE_ADDRESS};
use serde::{Deserialize, Serialize};

const MV_FLEXI_DAG_CONFIG_MODULE_NAME: &str = "FlexiDagConfig";
const MV_FLEXI_DAG_CONFIG_STRUCT_NAME: &str = "FlexiDagConfig";
const MV_FLEXI_DAG_CONFIG_STRUCT_NAME_V2: &str = "FlexiDagConfigV2";
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
            type_params: vec![],
        }))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct FlexiDagConfigV2 {
    pub pruning_depth: u64,
    pub pruning_finality: u64,
}

const G_PRUNING_DEPTH: u64 = 17280;
const G_PRUNING_FINALITY: u64 = 8640;

impl Default for FlexiDagConfigV2 {
    fn default() -> Self {
        Self {
            pruning_depth: G_PRUNING_DEPTH,
            pruning_finality: G_PRUNING_FINALITY,
        }
    }
}
impl OnChainConfig for FlexiDagConfigV2 {
    const MODULE_IDENTIFIER: &'static str = MV_FLEXI_DAG_CONFIG_MODULE_NAME;
    const CONF_IDENTIFIER: &'static str = MV_FLEXI_DAG_CONFIG_STRUCT_NAME_V2;
}

impl FlexiDagConfigV2 {
    pub fn type_tag() -> TypeTag {
        TypeTag::Struct(Box::new(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new(MV_FLEXI_DAG_CONFIG_MODULE_NAME).unwrap(),
            name: Identifier::new(MV_FLEXI_DAG_CONFIG_STRUCT_NAME_V2).unwrap(),
            type_params: vec![],
        }))
    }
    pub fn get_pruning_config(&self) -> (u64, u64) {
        (self.pruning_depth, self.pruning_finality)
    }
}
