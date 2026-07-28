// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! AI-readable Move module semantics types.
//!
//! These types expose deterministic, machine-readable metadata about VM2 Move
//! modules so that AI agents, wallets, indexers, and developer tools can
//! understand on-chain code without scraping prose or guessing from names.
//!
//! This module is gated behind the `ai-metadata` feature (disabled by default).
//! It does not change consensus, storage formats, transaction execution, or
//! production node behaviour.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Provenance – where a semantic fact originated
// ---------------------------------------------------------------------------

/// Identifies the source of a semantic fact so consumers can weigh its
/// reliability.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// Derived from compiled bytecode (high confidence).
    Bytecode,
    /// Derived from published ABI data.
    Abi,
    /// Derived from on-chain runtime metadata / attributes.
    RuntimeMetadata,
    /// Observed through a dry-run / execution preview.
    Preview,
    /// Declared by a developer or package manifest.
    Declared,
}

// ---------------------------------------------------------------------------
// Effect hints – conservative, source-annotated side-effect signals
// ---------------------------------------------------------------------------

/// A conservative hint about what a function *may* do.
///
/// These are not proofs of effect; when a fact cannot be determined
/// deterministically the metadata says `kind: "unknown"` with the appropriate
/// provenance.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct EffectHint {
    /// Machine-readable kind, e.g. `"writes_resources"`, `"reads_resources"`,
    /// `"emits_events"`, `"unknown"`.
    pub kind: String,
    /// Where this hint originated.
    pub source: Provenance,
}

impl EffectHint {
    pub fn new(kind: impl Into<String>, source: Provenance) -> Self {
        Self {
            kind: kind.into(),
            source,
        }
    }
}

// ---------------------------------------------------------------------------
// Runtime attributes
// ---------------------------------------------------------------------------

/// A key-value attribute attached to a module, function, or struct by the
/// Move compiler or runtime metadata system.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeAttribute {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl RuntimeAttribute {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
        }
    }

    pub fn with_value(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Some(value.into()),
        }
    }
}

// ---------------------------------------------------------------------------
// Struct / field semantics
// ---------------------------------------------------------------------------

/// AI-readable semantic view of a single struct field.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FieldSemantics {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    /// Where the field name and type were derived from.
    pub source: Provenance,
}

impl FieldSemantics {
    pub fn new(name: impl Into<String>, type_: impl Into<String>, source: Provenance) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
            source,
        }
    }
}

/// AI-readable declaration of one struct type parameter.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StructTypeParameterSemantics {
    /// Stable positional name because Move bytecode does not retain source names.
    pub name: String,
    /// Ability constraints in Move's canonical ability order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
    /// Whether the parameter was declared with `phantom`.
    pub is_phantom: bool,
    /// Where the parameter declaration was derived from.
    pub source: Provenance,
}

impl StructTypeParameterSemantics {
    pub fn new(
        name: impl Into<String>,
        constraints: Vec<String>,
        is_phantom: bool,
        source: Provenance,
    ) -> Self {
        Self {
            name: name.into(),
            constraints,
            is_phantom,
            source,
        }
    }
}

/// AI-readable semantic view of a Move struct definition.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StructSemantics {
    pub name: String,
    /// Abilities in Move's canonical `copy`, `drop`, `store`, `key` order.
    pub abilities: Vec<String>,
    /// Where `abilities` were derived from.
    pub abilities_source: Provenance,
    pub is_resource: bool,
    /// Where `is_resource` was derived from.
    pub is_resource_source: Provenance,
    /// Generic type parameters including constraints and phantom declarations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<StructTypeParameterSemantics>,
    pub fields: Vec<FieldSemantics>,
    pub attributes: Vec<RuntimeAttribute>,
}

// ---------------------------------------------------------------------------
// Function semantics
// ---------------------------------------------------------------------------

/// AI-readable semantic view of a Move function.
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FunctionSemantics {
    pub name: String,
    /// `"public"`, `"friend"`, `"private"`
    pub visibility: String,
    pub is_entry: bool,
    pub is_view: bool,
    /// Type parameter names (e.g. `["T0"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<String>,
    /// Human-readable parameter types (e.g. `["address", "u128"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<String>,
    /// Human-readable return types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub returns: Vec<String>,
    /// Conservative effect hints with provenance.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effect_hints: Vec<EffectHint>,
    /// Compiler/runtime attributes attached to this function.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<RuntimeAttribute>,
}

impl FunctionSemantics {
    pub fn new(name: impl Into<String>, visibility: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            visibility: visibility.into(),
            is_entry: false,
            is_view: false,
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            returns: Vec::new(),
            effect_hints: Vec::new(),
            attributes: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Module semantics – top-level container
// ---------------------------------------------------------------------------

/// AI-readable semantic view of a VM2 Move module.
///
/// This is the primary output type.  It is designed to be:
/// - deterministic (same module → same JSON)
/// - serializable (serde + JSON Schema via schemars)
/// - conservative (unknown facts are reported as limitations, not guessed)
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ModuleSemantics {
    /// Fully-qualified module identifier, e.g. `"0x1::Example"`.
    pub module: String,
    /// Hex-encoded SHA3-256 hash of the module bytecode.
    pub bytecode_hash: String,
    /// Hex-encoded SHA3-256 hash of the **interface surface** – stable across
    /// bytecode changes that do not alter the exposed API.
    pub interface_hash: String,
    /// Semantic view of every function in the module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<FunctionSemantics>,
    /// Semantic view of every struct in the module.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structs: Vec<StructSemantics>,
    /// Module-level runtime attributes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtime_attributes: Vec<RuntimeAttribute>,
    /// Known limitations of the current semantic extraction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
}

impl ModuleSemantics {
    pub fn new(
        module: impl Into<String>,
        bytecode_hash: impl Into<String>,
        interface_hash: impl Into<String>,
    ) -> Self {
        Self {
            module: module.into(),
            bytecode_hash: bytecode_hash.into(),
            interface_hash: interface_hash.into(),
            functions: Vec::new(),
            structs: Vec::new(),
            runtime_attributes: Vec::new(),
            limitations: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Hash helpers
// ---------------------------------------------------------------------------

/// Compute a SHA3-256 hash of `data` and return its hex encoding.
pub fn sha3_256_hex(data: &[u8]) -> String {
    use sha3::{Digest, Sha3_256};
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    format!("0x{}", hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha3_256_hex_empty() {
        let h = sha3_256_hex(b"");
        assert!(h.starts_with("0x"));
        assert_eq!(h.len(), 66); // "0x" + 64 hex chars
    }

    #[test]
    fn test_module_semantics_serde_roundtrip() {
        let ms = ModuleSemantics {
            module: "0x1::Foo".into(),
            bytecode_hash: "0xab".into(),
            interface_hash: "0xcd".into(),
            functions: vec![FunctionSemantics {
                name: "bar".into(),
                visibility: "public".into(),
                is_entry: true,
                is_view: false,
                type_parameters: vec!["T0".into()],
                parameters: vec!["address".into(), "u128".into()],
                returns: vec![],
                effect_hints: vec![EffectHint::new("writes_resources", Provenance::Preview)],
                attributes: vec![],
            }],
            structs: vec![StructSemantics {
                name: "Balance".into(),
                abilities: vec!["store".into(), "key".into()],
                abilities_source: Provenance::Bytecode,
                is_resource: true,
                is_resource_source: Provenance::Bytecode,
                type_parameters: vec![StructTypeParameterSemantics::new(
                    "T0",
                    vec!["store".into()],
                    true,
                    Provenance::Bytecode,
                )],
                fields: vec![FieldSemantics::new("value", "u128", Provenance::Bytecode)],
                attributes: vec![],
            }],
            runtime_attributes: vec![],
            limitations: vec!["read effects not fully inferred".into()],
        };

        let json = serde_json::to_string_pretty(&ms).unwrap();
        let back: ModuleSemantics = serde_json::from_str(&json).unwrap();
        assert_eq!(ms, back);
        assert!(json.contains(r#""abilities_source": "bytecode""#));
        assert!(json.contains(r#""is_phantom": true"#));
        assert!(json.contains(r#""source": "bytecode""#));
    }

    #[test]
    fn test_provenance_serde() {
        let p = Provenance::Bytecode;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, r#""bytecode""#);
        assert_eq!(serde_json::from_str::<Provenance>(&json).unwrap(), p);
    }

    #[test]
    fn test_function_semantics_defaults() {
        let f = FunctionSemantics::new("test", "public");
        assert!(!f.is_entry);
        assert!(!f.is_view);
        assert!(f.type_parameters.is_empty());
        assert!(f.parameters.is_empty());
        assert!(f.returns.is_empty());
        assert!(f.effect_hints.is_empty());
        assert!(f.attributes.is_empty());
    }
}
