// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::BuiltinNetworkID;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IdOrName {
    Id(u8),
    Name(String),
}

impl From<u8> for IdOrName {
    fn from(id: u8) -> Self {
        IdOrName::Id(id)
    }
}

impl From<&str> for IdOrName {
    fn from(name: &str) -> Self {
        IdOrName::Name(name.to_string().to_lowercase())
    }
}

static VM1_OFFLINE_HEIGHT: Lazy<HashMap<IdOrName, u64>> = Lazy::new(|| {
    let mut map: HashMap<IdOrName, u64> = HashMap::new();
    let mut update_height = |x: BuiltinNetworkID, height: u64| {
        let id = x.chain_id().id();
        let name = x.chain_name().to_lowercase();
        map.insert(id.into(), height);
        map.insert(name.as_str().into(), height);
    };
    // insert configured height for Builtin Network
    for x in BuiltinNetworkID::iter() {
        match x {
            BuiltinNetworkID::Main => {
                update_height(x, u64::MAX);
            }
            BuiltinNetworkID::Test => {
                update_height(x, u64::MAX);
            }
            BuiltinNetworkID::Dev => {
                update_height(x, u64::MAX);
            }
            BuiltinNetworkID::Halley => {
                update_height(x, u64::MAX);
            }
            BuiltinNetworkID::Proxima => {
                update_height(x, u64::MAX);
            }
            BuiltinNetworkID::Barnard => {
                update_height(x, u64::MAX);
            }
        }
    }
    // insert custom network height for test
    map.insert("vm2-only-testnet".into(), 0);
    map.insert("vm1-offline-testnet".into(), 3);
    map
});

pub fn vm1_offline_height(id_or_name: IdOrName) -> u64 {
    VM1_OFFLINE_HEIGHT
        .get(&id_or_name)
        .copied()
        .unwrap_or(u64::MAX)
}
