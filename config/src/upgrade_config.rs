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
        IdOrName::Name(name.to_lowercase())
    }
}

static VM1_OFFLINE_HEIGHT: Lazy<HashMap<IdOrName, u64>> = Lazy::new(|| {
    let mut map: HashMap<IdOrName, u64> = HashMap::new();
    let mut update_height = |x: BuiltinNetworkID, height: Option<u64>| {
        let name = x.chain_name().to_lowercase();
        let height = height.unwrap_or_else(|| {
            if let Ok(height_str) =
                std::env::var(format!("VM1_OFFLINE_HEIGHT_{}", name.to_uppercase()))
            {
                height_str.parse::<u64>().expect("invalid height")
            } else {
                u64::MAX
            }
        });
        let id = x.chain_id().id();
        assert!(height > 0, "vm1 offline height must be greater than 0");
        map.insert(id.into(), height);
        map.insert(name.as_str().into(), height);
    };
    // configured height for Builtin Network
    for x in BuiltinNetworkID::iter() {
        match x {
            BuiltinNetworkID::Dev => {
                update_height(x, None);
            }
            _ => {
                update_height(x, Some(u64::MAX));
            }
        }
    }
    let mut update_custom_network = |x: IdOrName, height: u64| {
        assert!(height > 0, "VM1 offline height must be greater than 0");
        assert!(!map.contains_key(&x), "Network id or name already exists");
        map.insert(x, height);
    };
    // custom network height for test
    // vm2-only-testnet
    update_custom_network(123.into(), 2);
    map
});

pub fn vm1_offline_height(id_or_name: IdOrName) -> u64 {
    VM1_OFFLINE_HEIGHT
        .get(&id_or_name)
        .copied()
        .unwrap_or(u64::MAX)
}
