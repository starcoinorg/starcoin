// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::TypeTag;
use crate::on_chain_config::OnChainConfig;
use move_core_types::account_address::AccountAddress;
use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use move_core_types::move_resource::{MoveResource, MoveStructType};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigChangeEvent<V: OnChainConfig> {
    pub account_address: AccountAddress,
    #[serde(deserialize_with = "V::deserialize")]
    pub config_value: V,
}

impl<V> ConfigChangeEvent<V> where V: OnChainConfig {}

impl<V> MoveStructType for ConfigChangeEvent<V>
where
    V: OnChainConfig + DeserializeOwned,
{
    const MODULE_NAME: &'static IdentStr = ident_str!("Config");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ConfigChangeEvent");
    fn type_args() -> Vec<TypeTag> {
        vec![TypeTag::Struct(Box::new(V::config_id().struct_tag()))]
    }
}

impl<V> MoveResource for ConfigChangeEvent<V> where V: OnChainConfig + DeserializeOwned {}
