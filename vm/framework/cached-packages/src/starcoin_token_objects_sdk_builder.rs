// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// This file was generated. Do not modify!
//
// To update this code, run: `cargo run --release -p framework`.

// Conversion library between a structured representation of a Move script call (`ScriptCall`) and the
// standard BCS-compatible representation used in Aptos transactions (`Script`).
//
// This code was generated by compiling known Script interfaces ("ABIs") with the tool `aptos-sdk-builder`.

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::get_first)]
use move_core_types::{
    ident_str,
    language_storage::{ModuleId, TypeTag},
};
use starcoin_vm_types::{
    account_address::AccountAddress,
    transaction::{ScriptFunction, TransactionPayload},
};

type Bytes = Vec<u8>;

/// Structured representation of a call into a known Move entry function.
/// ```ignore
/// impl EntryFunctionCall {
///     pub fn encode(self) -> TransactionPayload { .. }
///     pub fn decode(&TransactionPayload) -> Option<EntryFunctionCall> { .. }
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "fuzzing", proptest(no_params))]
pub enum EntryFunctionCall {
    /// Create a new collection
    AptosTokenCreateCollection {
        description: Vec<u8>,
        max_supply: u64,
        name: Vec<u8>,
        uri: Vec<u8>,
        mutable_description: bool,
        mutable_royalty: bool,
        mutable_uri: bool,
        mutable_token_description: bool,
        mutable_token_name: bool,
        mutable_token_properties: bool,
        mutable_token_uri: bool,
        tokens_burnable_by_creator: bool,
        tokens_freezable_by_creator: bool,
        royalty_numerator: u64,
        royalty_denominator: u64,
    },

    /// With an existing collection, directly mint a viable token into the creators account.
    AptosTokenMint {
        collection: Vec<u8>,
        description: Vec<u8>,
        name: Vec<u8>,
        uri: Vec<u8>,
        property_keys: Vec<Vec<u8>>,
        property_types: Vec<Vec<u8>>,
        property_values: Vec<Vec<u8>>,
    },

    /// With an existing collection, directly mint a soul bound token into the recipient's account.
    AptosTokenMintSoulBound {
        collection: Vec<u8>,
        description: Vec<u8>,
        name: Vec<u8>,
        uri: Vec<u8>,
        property_keys: Vec<Vec<u8>>,
        property_types: Vec<Vec<u8>>,
        property_values: Vec<Vec<u8>>,
        soul_bound_to: AccountAddress,
    },
}

impl EntryFunctionCall {
    /// Build an Aptos `TransactionPayload` from a structured object `EntryFunctionCall`.
    pub fn encode(self) -> TransactionPayload {
        use EntryFunctionCall::*;
        match self {
            AptosTokenCreateCollection {
                description,
                max_supply,
                name,
                uri,
                mutable_description,
                mutable_royalty,
                mutable_uri,
                mutable_token_description,
                mutable_token_name,
                mutable_token_properties,
                mutable_token_uri,
                tokens_burnable_by_creator,
                tokens_freezable_by_creator,
                royalty_numerator,
                royalty_denominator,
            } => aptos_token_create_collection(
                description,
                max_supply,
                name,
                uri,
                mutable_description,
                mutable_royalty,
                mutable_uri,
                mutable_token_description,
                mutable_token_name,
                mutable_token_properties,
                mutable_token_uri,
                tokens_burnable_by_creator,
                tokens_freezable_by_creator,
                royalty_numerator,
                royalty_denominator,
            ),
            AptosTokenMint {
                collection,
                description,
                name,
                uri,
                property_keys,
                property_types,
                property_values,
            } => aptos_token_mint(
                collection,
                description,
                name,
                uri,
                property_keys,
                property_types,
                property_values,
            ),
            AptosTokenMintSoulBound {
                collection,
                description,
                name,
                uri,
                property_keys,
                property_types,
                property_values,
                soul_bound_to,
            } => aptos_token_mint_soul_bound(
                collection,
                description,
                name,
                uri,
                property_keys,
                property_types,
                property_values,
                soul_bound_to,
            ),
        }
    }

    /// Try to recognize an Aptos `TransactionPayload` and convert it into a structured object `EntryFunctionCall`.
    pub fn decode(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::ScriptFunction(script) = payload {
            match SCRIPT_FUNCTION_DECODER_MAP.get(&format!(
                "{}_{}",
                script.module().name(),
                script.function()
            )) {
                Some(decoder) => decoder(payload),
                None => None,
            }
        } else {
            None
        }
    }
}

/// Create a new collection
pub fn aptos_token_create_collection(
    description: Vec<u8>,
    max_supply: u64,
    name: Vec<u8>,
    uri: Vec<u8>,
    mutable_description: bool,
    mutable_royalty: bool,
    mutable_uri: bool,
    mutable_token_description: bool,
    mutable_token_name: bool,
    mutable_token_properties: bool,
    mutable_token_uri: bool,
    tokens_burnable_by_creator: bool,
    tokens_freezable_by_creator: bool,
    royalty_numerator: u64,
    royalty_denominator: u64,
) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        ModuleId::new(
            AccountAddress::new([
                // 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                // 0, 0, 0, 4,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4,
            ]),
            ident_str!("aptos_token").to_owned(),
        ),
        ident_str!("create_collection").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&description).unwrap(),
            bcs::to_bytes(&max_supply).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&uri).unwrap(),
            bcs::to_bytes(&mutable_description).unwrap(),
            bcs::to_bytes(&mutable_royalty).unwrap(),
            bcs::to_bytes(&mutable_uri).unwrap(),
            bcs::to_bytes(&mutable_token_description).unwrap(),
            bcs::to_bytes(&mutable_token_name).unwrap(),
            bcs::to_bytes(&mutable_token_properties).unwrap(),
            bcs::to_bytes(&mutable_token_uri).unwrap(),
            bcs::to_bytes(&tokens_burnable_by_creator).unwrap(),
            bcs::to_bytes(&tokens_freezable_by_creator).unwrap(),
            bcs::to_bytes(&royalty_numerator).unwrap(),
            bcs::to_bytes(&royalty_denominator).unwrap(),
        ],
    ))
}

/// With an existing collection, directly mint a viable token into the creators account.
pub fn aptos_token_mint(
    collection: Vec<u8>,
    description: Vec<u8>,
    name: Vec<u8>,
    uri: Vec<u8>,
    property_keys: Vec<Vec<u8>>,
    property_types: Vec<Vec<u8>>,
    property_values: Vec<Vec<u8>>,
) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        ModuleId::new(
            AccountAddress::new([
                // 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                // 0, 0, 0, 4,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4,
            ]),
            ident_str!("aptos_token").to_owned(),
        ),
        ident_str!("mint").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&description).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&uri).unwrap(),
            bcs::to_bytes(&property_keys).unwrap(),
            bcs::to_bytes(&property_types).unwrap(),
            bcs::to_bytes(&property_values).unwrap(),
        ],
    ))
}

/// With an existing collection, directly mint a soul bound token into the recipient's account.
pub fn aptos_token_mint_soul_bound(
    collection: Vec<u8>,
    description: Vec<u8>,
    name: Vec<u8>,
    uri: Vec<u8>,
    property_keys: Vec<Vec<u8>>,
    property_types: Vec<Vec<u8>>,
    property_values: Vec<Vec<u8>>,
    soul_bound_to: AccountAddress,
) -> TransactionPayload {
    TransactionPayload::ScriptFunction(ScriptFunction::new(
        ModuleId::new(
            AccountAddress::new([
                // 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                // 0, 0, 0, 4,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4,
            ]),
            ident_str!("aptos_token").to_owned(),
        ),
        ident_str!("mint_soul_bound").to_owned(),
        vec![],
        vec![
            bcs::to_bytes(&collection).unwrap(),
            bcs::to_bytes(&description).unwrap(),
            bcs::to_bytes(&name).unwrap(),
            bcs::to_bytes(&uri).unwrap(),
            bcs::to_bytes(&property_keys).unwrap(),
            bcs::to_bytes(&property_types).unwrap(),
            bcs::to_bytes(&property_values).unwrap(),
            bcs::to_bytes(&soul_bound_to).unwrap(),
        ],
    ))
}
mod decoder {
    use super::*;
    pub fn aptos_token_create_collection(
        payload: &TransactionPayload,
    ) -> Option<EntryFunctionCall> {
        if let TransactionPayload::ScriptFunction(script) = payload {
            Some(EntryFunctionCall::AptosTokenCreateCollection {
                description: bcs::from_bytes(script.args().get(0)?).ok()?,
                max_supply: bcs::from_bytes(script.args().get(1)?).ok()?,
                name: bcs::from_bytes(script.args().get(2)?).ok()?,
                uri: bcs::from_bytes(script.args().get(3)?).ok()?,
                mutable_description: bcs::from_bytes(script.args().get(4)?).ok()?,
                mutable_royalty: bcs::from_bytes(script.args().get(5)?).ok()?,
                mutable_uri: bcs::from_bytes(script.args().get(6)?).ok()?,
                mutable_token_description: bcs::from_bytes(script.args().get(7)?).ok()?,
                mutable_token_name: bcs::from_bytes(script.args().get(8)?).ok()?,
                mutable_token_properties: bcs::from_bytes(script.args().get(9)?).ok()?,
                mutable_token_uri: bcs::from_bytes(script.args().get(10)?).ok()?,
                tokens_burnable_by_creator: bcs::from_bytes(script.args().get(11)?).ok()?,
                tokens_freezable_by_creator: bcs::from_bytes(script.args().get(12)?).ok()?,
                royalty_numerator: bcs::from_bytes(script.args().get(13)?).ok()?,
                royalty_denominator: bcs::from_bytes(script.args().get(14)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn aptos_token_mint(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::ScriptFunction(script) = payload {
            Some(EntryFunctionCall::AptosTokenMint {
                collection: bcs::from_bytes(script.args().get(0)?).ok()?,
                description: bcs::from_bytes(script.args().get(1)?).ok()?,
                name: bcs::from_bytes(script.args().get(2)?).ok()?,
                uri: bcs::from_bytes(script.args().get(3)?).ok()?,
                property_keys: bcs::from_bytes(script.args().get(4)?).ok()?,
                property_types: bcs::from_bytes(script.args().get(5)?).ok()?,
                property_values: bcs::from_bytes(script.args().get(6)?).ok()?,
            })
        } else {
            None
        }
    }

    pub fn aptos_token_mint_soul_bound(payload: &TransactionPayload) -> Option<EntryFunctionCall> {
        if let TransactionPayload::ScriptFunction(script) = payload {
            Some(EntryFunctionCall::AptosTokenMintSoulBound {
                collection: bcs::from_bytes(script.args().get(0)?).ok()?,
                description: bcs::from_bytes(script.args().get(1)?).ok()?,
                name: bcs::from_bytes(script.args().get(2)?).ok()?,
                uri: bcs::from_bytes(script.args().get(3)?).ok()?,
                property_keys: bcs::from_bytes(script.args().get(4)?).ok()?,
                property_types: bcs::from_bytes(script.args().get(5)?).ok()?,
                property_values: bcs::from_bytes(script.args().get(6)?).ok()?,
                soul_bound_to: bcs::from_bytes(script.args().get(7)?).ok()?,
            })
        } else {
            None
        }
    }
}

type EntryFunctionDecoderMap = std::collections::HashMap<
    String,
    Box<
        dyn Fn(&TransactionPayload) -> Option<EntryFunctionCall>
            + std::marker::Sync
            + std::marker::Send,
    >,
>;

static SCRIPT_FUNCTION_DECODER_MAP: once_cell::sync::Lazy<EntryFunctionDecoderMap> =
    once_cell::sync::Lazy::new(|| {
        let mut map: EntryFunctionDecoderMap = std::collections::HashMap::new();
        map.insert(
            "aptos_token_create_collection".to_string(),
            Box::new(decoder::aptos_token_create_collection),
        );
        map.insert(
            "aptos_token_mint".to_string(),
            Box::new(decoder::aptos_token_mint),
        );
        map.insert(
            "aptos_token_mint_soul_bound".to_string(),
            Box::new(decoder::aptos_token_mint_soul_bound),
        );
        map
    });
