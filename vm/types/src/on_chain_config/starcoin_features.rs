// Copyright (c) The Starcoin Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::OnChainConfig;
use serde::{Deserialize, Serialize};

/// The feature flags define in the Move source. This must stay aligned with the constants there.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum FeatureFlag {
    VM_BINARY_FORMAT_V6 = 1,
}

/// Representation of features on chain as a bitset.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Features {
    #[serde(with = "serde_bytes")]
    pub features: Vec<u8>,
}

impl Default for Features {
    fn default() -> Self {
        let mut features = Features {
            features: vec![0; 5],
        };

        use FeatureFlag::*;
        features.enable(VM_BINARY_FORMAT_V6);
        features
    }
}

impl OnChainConfig for Features {
    const MODULE_IDENTIFIER: &'static str = "features";
    const CONF_IDENTIFIER: &'static str = "Features";
}

impl Features {
    pub fn enable(&mut self, flag: FeatureFlag) {
        let byte_index = (flag as u64 / 8) as usize;
        let bit_mask = 1 << (flag as u64 % 8);
        while self.features.len() <= byte_index {
            self.features.push(0);
        }

        self.features[byte_index] |= bit_mask;
    }

    pub fn is_enabled(&self, flag: FeatureFlag) -> bool {
        let val = flag as u64;
        let byte_index = (val / 8) as usize;
        let bit_mask = 1 << (val % 8);
        byte_index < self.features.len() && (self.features[byte_index] & bit_mask != 0)
    }

    pub fn is_vm_binary_format_v6_enabled(&self) -> bool {
        self.is_enabled(FeatureFlag::VM_BINARY_FORMAT_V6)
    }
}