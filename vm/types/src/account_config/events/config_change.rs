// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::TypeTag;
use crate::on_chain_config::OnChainConfig;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::MoveStructType;
use serde::{Deserializer, Serialize};

//TODO support deserialize
#[derive(Debug, Serialize)]
pub struct ConfigChangeEvent<V: OnChainConfig> {
    pub account_address: AccountAddress,
    pub config_value: V,
}

impl<V> ConfigChangeEvent<V> where V: OnChainConfig {}

impl<V: OnChainConfig> MoveStructType for ConfigChangeEvent<V> {
    const MODULE_NAME: &'static IdentStr = ident_str!("Config");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ConfigChangeEvent");
    fn type_args() -> Vec<TypeTag> {
        vec![TypeTag::Struct(Box::new(V::config_id().struct_tag()))]
    }
}
