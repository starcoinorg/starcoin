// Copyright © Starcoin Foundation
// SPDX-License-Identifier: Apache-2.0

// See in https://github.com/aptos-labs/aptos-core/blob/05c0aa0090111328ced569c5733444869631c0b8/aptos-move/framework/src/module_metadata.rs#L2

use move_core_types::{errmap::ErrorDescription, language_storage::StructTag};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The minimal file format version from which the V1 metadata is supported
pub const METADATA_V1_MIN_FILE_FORMAT_VERSION: u32 = 6;

/// The keys used to identify the metadata in the metadata section of the module bytecode.
/// This is more or less arbitrary, besides we should use some unique key to identify
/// Starcoin specific metadata (`starcoin::` here).
pub static STARCOIN_METADATA_KEY: &[u8] = "starcoin::metadata_v0".as_bytes();
pub static STARCOIN_METADATA_KEY_V1: &[u8] = "starcoin::metadata_v1".as_bytes();

/// Starcoin specific metadata attached to the metadata section of file_format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeModuleMetadata {
    /// The error map containing the description of error reasons as grabbed from the source.
    /// These are typically only a few entries so no relevant size difference.
    pub error_map: BTreeMap<u64, ErrorDescription>,
}

/// V1 of Starcoin specific metadata attached to the metadata section of file_format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeModuleMetadataV1 {
    /// The error map containing the description of error reasons as grabbed from the source.
    /// These are typically only a few entries so no relevant size difference.
    pub error_map: BTreeMap<u64, ErrorDescription>,

    /// Attributes attached to structs.
    pub struct_attributes: BTreeMap<String, Vec<KnownAttribute>>,

    /// Attributes attached to functions, by definition index.
    pub fun_attributes: BTreeMap<String, Vec<KnownAttribute>>,
}

/// Enumeration of potentially known attributes
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KnownAttribute {
    kind: u8,
    args: Vec<String>,
}

/// Enumeration of known attributes
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KnownAttributeKind {
    // An older compiler placed view functions at 0. This was then published to
    // Testnet, and now we need to recognize this as a legacy index.
    LegacyViewFunction = 0,
    ViewFunction = 1,
    ResourceGroup = 2,
    ResourceGroupMember = 3,
    Event = 4,
    Randomness = 5,
}

impl KnownAttribute {
    pub fn view_function() -> Self {
        Self {
            kind: KnownAttributeKind::ViewFunction as u8,
            args: vec![],
        }
    }

    pub fn is_view_function(&self) -> bool {
        self.kind == (KnownAttributeKind::LegacyViewFunction as u8)
            || self.kind == (KnownAttributeKind::ViewFunction as u8)
    }

    pub fn is_resource_group(&self) -> bool {
        self.kind == KnownAttributeKind::ResourceGroup as u8
    }

    pub fn resource_group_member(container: String) -> Self {
        Self {
            kind: KnownAttributeKind::ResourceGroupMember as u8,
            args: vec![container],
        }
    }

    pub fn get_resource_group_member(&self) -> Option<StructTag> {
        if self.kind == KnownAttributeKind::ResourceGroupMember as u8 {
            self.args.first()?.parse().ok()
        } else {
            None
        }
    }

    pub fn is_resource_group_member(&self) -> bool {
        self.kind == KnownAttributeKind::ResourceGroupMember as u8
    }

    pub fn event() -> Self {
        Self {
            kind: KnownAttributeKind::Event as u8,
            args: vec![],
        }
    }

    pub fn is_event(&self) -> bool {
        self.kind == KnownAttributeKind::Event as u8
    }

    pub fn randomness(claimed_gas: Option<u64>) -> Self {
        Self {
            kind: KnownAttributeKind::Randomness as u8,
            args: if let Some(amount) = claimed_gas {
                vec![amount.to_string()]
            } else {
                vec![]
            },
        }
    }

    pub fn is_randomness(&self) -> bool {
        self.kind == KnownAttributeKind::Randomness as u8
    }
}
