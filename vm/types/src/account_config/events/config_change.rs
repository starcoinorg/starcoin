// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::language_storage::TypeTag;
use crate::move_resource::MoveResource;
use crate::on_chain_config::OnChainConfig;
use move_core_types::account_address::AccountAddress;
use serde::Serialize;

//TODO support deserialize
#[derive(Debug, Serialize)]
pub struct ConfigChangeEvent<V: OnChainConfig> {
    pub account_address: AccountAddress,
    pub config_value: V,
}

impl<V> ConfigChangeEvent<V> where V: OnChainConfig {}

impl<V> MoveResource for ConfigChangeEvent<V>
where
    V: OnChainConfig,
{
    const MODULE_NAME: &'static str = "Config";
    const STRUCT_NAME: &'static str = "ConfigChangeEvent";

    fn type_params() -> Vec<TypeTag> {
        vec![TypeTag::Struct(V::config_id().struct_tag())]
    }
}
