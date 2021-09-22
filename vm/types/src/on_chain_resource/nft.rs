// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::account_config::CORE_CODE_ADDRESS;
use crate::language_storage::{StructTag, TypeTag};
use crate::move_resource::MoveResource;
use crate::parser::parse_type_tag;
use anyhow::{ensure, format_err, Result};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct NFTType {
    pub meta_type: TypeTag,
    pub body_type: TypeTag,
}

impl Display for NFTType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.meta_type, self.body_type)
    }
}

impl FromStr for NFTType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('/').collect::<Vec<_>>();
        ensure!(
            parts.len() == 2,
            "NFTType format, expect: meta_type/body_type, but got: {}",
            s
        );
        let meta_type = parse_type_tag(parts[0])?;
        let body_type = parse_type_tag(parts[1])?;
        Ok(NFTType {
            meta_type,
            body_type,
        })
    }
}

impl<'de> Deserialize<'de> for NFTType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            NFTType::from_str(&s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "NFTType")]
            struct Value {
                meta_type: TypeTag,
                body_type: TypeTag,
            }

            let value = Value::deserialize(deserializer)?;
            Ok(NFTType {
                meta_type: value.meta_type,
                body_type: value.body_type,
            })
        }
    }
}

impl Serialize for NFTType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct(
                "NFTType",
                &(self.meta_type.clone(), self.body_type.clone()),
            )
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct NFTUUID {
    pub nft_type: NFTType,
    pub id: u64,
}

impl Display for NFTUUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.nft_type, self.id)
    }
}

impl FromStr for NFTUUID {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let idx = s.rfind('/').ok_or_else(|| {
            format_err!("Invalid NFTUUID format, expect nft_type/id, got : {}", s)
        })?;
        let (nft_type_str, id_str) = s.split_at(idx);
        let nft_type = NFTType::from_str(nft_type_str)?;
        let id = id_str.strip_prefix('/').unwrap_or(id_str).parse()?;
        Ok(NFTUUID { nft_type, id })
    }
}

impl<'de> Deserialize<'de> for NFTUUID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            NFTUUID::from_str(&s).map_err(D::Error::custom)
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize)]
            #[serde(rename = "NFTUUID")]
            struct Value {
                nft_type: NFTType,
                id: u64,
            }

            let value = Value::deserialize(deserializer)?;
            Ok(NFTUUID {
                nft_type: value.nft_type,
                id: value.id,
            })
        }
    }
}

impl Serialize for NFTUUID {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            // See comment in deserialize.
            serializer.serialize_newtype_struct("NFTUUID", &(self.nft_type.clone(), self.id))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Metadata {
    pub name: String,
    /// Image link, such as ipfs://xxxx
    pub image: String,
    /// Image bytes data, hex format, image or image_data can not empty for both.
    pub image_data: String,
    /// NFT description utf8 bytes.
    pub description: String,
}

impl Metadata {
    pub fn from_json(json_value: serde_json::Value) -> Result<Metadata> {
        let meta_json = json_value
            .as_object()
            .ok_or_else(|| format_err!("expect a json object, but got : {:?}", json_value))?;
        Ok(Self {
            name: meta_json
                .get("name")
                .ok_or_else(|| format_err!("invalid json, miss name field: {}", json_value))
                .and_then(|value| {
                    value
                        .as_str()
                        .ok_or_else(|| format_err!("expect str but got {}", value))
                })
                .and_then(|hex_str| {
                    hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str)).map_err(|e| e.into())
                })
                .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.into()))?,
            image: meta_json
                .get("image")
                .ok_or_else(|| format_err!("invalid json, miss image field, {}", json_value))
                .and_then(|value| {
                    value
                        .as_str()
                        .ok_or_else(|| format_err!("expect str but got {}", value))
                })
                .and_then(|hex_str| {
                    hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str)).map_err(|e| e.into())
                })
                .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.into()))?,
            image_data: meta_json
                .get("image_data")
                .ok_or_else(|| format_err!("invalid json, miss image_data field, {}", json_value))
                .and_then(|value| {
                    value
                        .as_str()
                        .map(|s| s.to_string())
                        .ok_or_else(|| format_err!("expect str but got {}", value))
                })?,
            description: meta_json
                .get("description")
                .ok_or_else(|| format_err!("invalid json, miss description field, {}", json_value))
                .and_then(|value| {
                    value
                        .as_str()
                        .ok_or_else(|| format_err!("expect str but got {}", value))
                })
                .and_then(|hex_str| {
                    hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str)).map_err(|e| e.into())
                })
                .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.into()))?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct NFT {
    pub nft_type: NFTType,
    pub creator: AccountAddress,
    pub id: u64,
    pub base_meta: Metadata,
    pub type_meta: serde_json::Value,
    pub body: serde_json::Value,
}

impl NFT {
    pub fn from_json(nft_type: NFTType, json_value: serde_json::Value) -> Result<NFT> {
        let nft_json = json_value
            .as_object()
            .ok_or_else(|| format_err!("expect a json object, but got : {:?}", json_value))?;
        Ok(Self {
            nft_type,
            creator: nft_json
                .get("creator")
                .and_then(|creator_json| creator_json.as_str())
                .and_then(|addr| AccountAddress::from_str(addr).ok())
                .ok_or_else(|| format_err!("invalid json, parse creator failed, {}", json_value))?,
            id: nft_json
                .get("id")
                .and_then(|id_json| id_json.as_u64())
                .ok_or_else(|| format_err!("invalid json, parse id failed, {}", json_value))?,
            base_meta: nft_json
                .get("base_meta")
                .ok_or_else(|| format_err!("miss base_meta field, {}", json_value))
                .and_then(|base_meta| Metadata::from_json(base_meta.clone()))?,

            type_meta: nft_json.get("type_meta").cloned().ok_or_else(|| {
                format_err!("invalid json, parse type_meta failed, {}", json_value)
            })?,
            body: nft_json
                .get("body")
                .cloned()
                .ok_or_else(|| format_err!("invalid json, parse body failed, {}", json_value))?,
        })
    }

    pub fn uuid(&self) -> NFTUUID {
        NFTUUID {
            nft_type: self.nft_type.clone(),
            id: self.id,
        }
    }
}

impl MoveResource for NFT {
    const MODULE_NAME: &'static str = "NFT";
    const STRUCT_NAME: &'static str = "NFT";
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct NFTGallery {
    pub items: Vec<NFT>,
}

impl NFTGallery {
    /// Get nft type from NFTGallery, return None if struct tag is not a valid NFTGallery StructTag
    pub fn nft_type(struct_tag: &StructTag) -> Option<NFTType> {
        if struct_tag.address == CORE_CODE_ADDRESS
            && struct_tag.module.as_str() == Self::MODULE_NAME
            && struct_tag.name.as_str() == Self::STRUCT_NAME
        {
            if struct_tag.type_params.len() == 2 {
                let (meta_type, body_type) = (
                    struct_tag.type_params.get(0).cloned().unwrap(),
                    struct_tag.type_params.get(1).cloned().unwrap(),
                );
                Some(NFTType {
                    meta_type,
                    body_type,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn from_json(nft_type: NFTType, json_value: serde_json::Value) -> Result<NFTGallery> {
        let gallery_json = json_value
            .as_object()
            .ok_or_else(|| format_err!("expect a json object, but got : {:?}", json_value))?;
        let items = gallery_json
            .get("items")
            .and_then(|items_json| items_json.as_array().cloned())
            .ok_or_else(|| format_err!("invalid json, parse items failed, {}", json_value))?;
        let items: Result<Vec<NFT>> = items
            .into_iter()
            .map(|item| NFT::from_json(nft_type.clone(), item))
            .collect();
        Ok(NFTGallery { items: items? })
    }
}

impl MoveResource for NFTGallery {
    const MODULE_NAME: &'static str = "NFTGallery";
    const STRUCT_NAME: &'static str = "NFTGallery";
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[allow(clippy::upper_case_acronyms)]
pub struct IdentifierNFT {
    nft_type: NFTType,
    nft: Option<NFT>,
}

impl IdentifierNFT {
    /// Get nft type from IdentifierNFT, return None if struct tag is not a valid IdentifierNFT StructTag
    pub fn nft_type(struct_tag: &StructTag) -> Option<NFTType> {
        if struct_tag.address == CORE_CODE_ADDRESS
            && struct_tag.module.as_str() == Self::MODULE_NAME
            && struct_tag.name.as_str() == Self::STRUCT_NAME
        {
            if struct_tag.type_params.len() == 2 {
                let (meta_type, body_type) = (
                    struct_tag.type_params.get(0).cloned().unwrap(),
                    struct_tag.type_params.get(1).cloned().unwrap(),
                );
                Some(NFTType {
                    meta_type,
                    body_type,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn from_json(nft_type: NFTType, json_value: serde_json::Value) -> Result<IdentifierNFT> {
        let ident_json = json_value
            .as_object()
            .ok_or_else(|| format_err!("expect a json object, but got : {:?}", json_value))?;
        // the Option<T> in Move is vec<T>
        let nft = ident_json
            .get("nft")
            .ok_or_else(|| format_err!("invalid json, miss nft field, {}", json_value))
            .and_then(|nft_json| {
                nft_json
                    .as_object()
                    .ok_or_else(|| format_err!("expect nft as json object, but got: {}", nft_json))?
                    .get("vec")
                    .ok_or_else(|| format_err!("invalid json, miss vec field, {}", nft_json))
            })
            .and_then(|nft_vec| {
                nft_vec.as_array().cloned().ok_or_else(|| {
                    format_err!("invalid json, expect vec field, but got {}", nft_vec)
                })
            })?
            .pop()
            .map(|nft_value| NFT::from_json(nft_type.clone(), nft_value))
            .transpose()?;

        Ok(IdentifierNFT { nft_type, nft })
    }
}

impl MoveResource for IdentifierNFT {
    const MODULE_NAME: &'static str = "IdentifierNFT";
    const STRUCT_NAME: &'static str = "IdentifierNFT";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid() {
        let uuid = "0x1::GenesisNFT::GenesisNFTMeta/0x1::GenesisNFT::GenesisNFT/1";
        let uuid = NFTUUID::from_str(uuid).unwrap();
        assert_eq!(uuid.id, 1);
    }
}
