// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_config::genesis_address;
use crate::state_view::StateView;
use crate::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::CORE_CODE_ADDRESS,
    event::EventKey,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use anyhow::{format_err, Result};
use bytes::Bytes;
use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use std::{collections::HashMap, sync::Arc};

mod consensus_config;
mod dao_config;
mod starcoin_features;

mod flexi_dag_config;
mod gas_schedule;
mod genesis_gas_schedule;
mod move_lang_version;
mod timestamp;
mod version;
mod vm_config;

mod timed_features;

pub use self::{
    consensus_config::{consensus_config_type_tag, ConsensusConfig, G_CONSENSUS_CONFIG_IDENTIFIER},
    dao_config::DaoConfig,
    flexi_dag_config::*,
    gas_schedule::{
        instruction_gas_schedule_v1, instruction_gas_schedule_v2, native_gas_schedule_v1,
        native_gas_schedule_v2, native_gas_schedule_v3, native_gas_schedule_v4,
        txn_gas_schedule_test, txn_gas_schedule_v1, txn_gas_schedule_v2, txn_gas_schedule_v3,
        GasSchedule, G_GAS_SCHEDULE_GAS_SCHEDULE, G_GAS_SCHEDULE_IDENTIFIER,
    },
    genesis_gas_schedule::{
        instruction_table_v1, instruction_table_v2, native_table_v1, native_table_v2,
        v4_native_table, G_LATEST_INSTRUCTION_TABLE, G_LATEST_NATIVE_TABLE,
    },
    move_lang_version::MoveLanguageVersion,
    timestamp::CurrentTimeMicroseconds,
    version::{version_config_type_tag, Version, G_VERSION_CONFIG_IDENTIFIER},
    vm_config::*,
    starcoin_features::*,
    timed_features::*,
};
pub use crate::on_chain_resource::GlobalTimeOnChain;
use crate::state_store::state_key::StateKey;

/// To register an on-chain config in Rust:
/// 1. Implement the `OnChainConfig` trait for the Rust representation of the config
/// 2. Add the config's `ConfigID` to `ON_CHAIN_CONFIG_REGISTRY`
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[allow(clippy::box_collection)]
#[allow(clippy::upper_case_acronyms)]
pub struct ConfigID(&'static str, &'static str, &'static str, Vec<TypeTag>);

impl ConfigID {
    pub fn access_path(self) -> AccessPath {
        access_path_for_config(
            AccountAddress::from_hex_literal(self.0).expect("failed to get address"),
            Identifier::new(self.1).expect("failed to get Identifier"),
            Identifier::new(self.2).expect("failed to get Identifier"),
            self.3,
        )
    }

    pub fn struct_tag(self) -> StructTag {
        StructTag {
            address: AccountAddress::from_hex_literal(self.0).expect("failed to get address"),
            module: Identifier::new(self.1).expect("failed to get Identifier"),
            name: Identifier::new(self.2).expect("failed to get Identifier"),
            type_args: self.3,
        }
    }
}

pub static G_ON_CHAIN_CONFIG_REGISTRY: Lazy<Vec<ConfigID>> = Lazy::new(|| {
    vec![
        Version::config_id(),
        ConsensusConfig::config_id(),
        DaoConfig::config_id(),
    ]
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnChainConfigPayload {
    epoch: u64,
    configs: Arc<HashMap<ConfigID, Vec<u8>>>,
}

impl OnChainConfigPayload {
    pub fn new(epoch: u64, configs: Arc<HashMap<ConfigID, Vec<u8>>>) -> Self {
        Self { epoch, configs }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn get<T: OnChainConfig>(&self) -> Result<T> {
        let bytes = self
            .configs
            .get(&T::config_id())
            .ok_or_else(|| format_err!("[on-chain cfg] config not in payload"))?;
        T::deserialize_into_config(bytes)
    }

    pub fn configs(&self) -> &HashMap<ConfigID, Vec<u8>> {
        &self.configs
    }
}

/// Trait to be implemented by a storage type from which to read on-chain configs
pub trait ConfigStorage {
    fn fetch_config(&self, access_path: AccessPath) -> Option<Bytes>;
}

impl<V> ConfigStorage for V
where
    V: StateView,
{
    fn fetch_config(&self, access_path: AccessPath) -> Option<Bytes> {
        self.get_state_value(&StateKey::AccessPath(access_path))
            .ok()?
            .map(|s| s.bytes().clone())
    }
}

/// Trait to be implemented by a Rust struct representation of an on-chain config
/// that is stored in storage as a serialized byte array
pub trait OnChainConfig: Send + Sync + DeserializeOwned {
    const ADDRESS: &'static str = "0x1";
    const MODULE_IDENTIFIER: &'static str;
    const CONF_IDENTIFIER: &'static str;

    // Single-round LCS deserialization from bytes to `Self`
    // This is the expected deserialization pattern for most Rust representations,
    // but sometimes `deserialize_into_config` may need an extra customized round of deserialization
    // (e.g. enums like `VMPublishingOption`)
    // In the override, we can reuse this default logic via this function
    // Note: we cannot directly call the default `deserialize_into_config` implementation
    // in its override - this will just refer to the override implementation itself
    fn deserialize_default_impl(bytes: &[u8]) -> Result<Self> {
        bcs_ext::from_bytes::<Self>(bytes)
            .map_err(|e| format_err!("[on-chain config] Failed to deserialize into config: {}", e))
    }

    // Function for deserializing bytes to `Self`
    // It will by default try one round of LCS deserialization directly to `Self`
    // The implementation for the concrete type should override this function if this
    // logic needs to be customized
    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        Self::deserialize_default_impl(bytes)
    }

    fn fetch_config<T>(storage: &T) -> Result<Option<Self>>
    where
        T: ConfigStorage,
    {
        storage
            .fetch_config(Self::config_id().access_path())
            .map(|bytes| Self::deserialize_into_config(&bytes))
            .transpose()
    }

    fn type_params() -> Vec<TypeTag> {
        vec![]
    }

    fn config_id() -> ConfigID {
        ConfigID(
            Self::ADDRESS,
            Self::MODULE_IDENTIFIER,
            Self::CONF_IDENTIFIER,
            Self::type_params(),
        )
    }
}

pub fn new_epoch_event_key() -> EventKey {
    EventKey::new_from_address(&genesis_address(), 0)
}

pub fn access_path_for_config(
    address: AccountAddress,
    module_name: Identifier,
    config_name: Identifier,
    params: Vec<TypeTag>,
) -> AccessPath {
    AccessPath::resource_access_path(
        address,
        StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("Config").unwrap(),
            name: Identifier::new("Config").unwrap(),
            type_args: vec![TypeTag::Struct(Box::new(StructTag {
                address: CORE_CODE_ADDRESS,
                module: module_name,
                name: config_name,
                type_args: params,
            }))],
        },
    )
}
