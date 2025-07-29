// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

const MV_LANG_VERSION_MODULE_NAME: &str = "LanguageVersion";
const MV_LANG_VERSION_STRUCT_NAME: &str = "LanguageVersion";

/// Defines the move language version.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct MoveLanguageVersion {
    pub major: u64,
}

impl OnChainConfig for MoveLanguageVersion {
    const MODULE_IDENTIFIER: &'static str = MV_LANG_VERSION_MODULE_NAME;
    const CONF_IDENTIFIER: &'static str = MV_LANG_VERSION_STRUCT_NAME;
}
impl MoveResource for MoveLanguageVersion {
    const MODULE_NAME: &'static str = MV_LANG_VERSION_MODULE_NAME;
    const STRUCT_NAME: &'static str = MV_LANG_VERSION_STRUCT_NAME;
}
