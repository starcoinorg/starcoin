// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! AI-readable Move module semantics resolver.
//!
//! Derives [`ModuleSemantics`] from compiled Move modules using the existing
//! VM2 resolver infrastructure.  Only deterministic, bytecode-derived and
//! ABI-derived facts are included; when a field cannot be determined it is
//! omitted or reported as a limitation.
//!
//! Gated behind the `ai-metadata` feature (disabled by default).

use anyhow::Result;
#[cfg(feature = "ai-metadata")]
use bcs;
use starcoin_vm2_abi_types::ai_semantics::{
    sha3_256_hex, EffectHint, FieldSemantics, FunctionSemantics, ModuleSemantics, Provenance,
    StructSemantics, StructTypeParameterSemantics,
};
use starcoin_vm2_resource_viewer::module_cache::ModuleCache;
use starcoin_vm2_resource_viewer::resolver::Resolver;
use starcoin_vm2_vm_types::access::ModuleAccess;
use starcoin_vm2_vm_types::file_format::{
    AbilitySet, CompiledModule, Visibility,
};
use starcoin_vm2_vm_types::identifier::{IdentStr, Identifier};
use starcoin_vm2_vm_types::language_storage::ModuleId;
use starcoin_vm2_vm_types::normalized::{Function, Module, Struct};
use starcoin_vm2_vm_types::state_store::StateView;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Minimal metadata types for module attribute extraction
// ---------------------------------------------------------------------------

/// Key used to store V1 runtime metadata in the compiled module's metadata section.
const STARCOIN_METADATA_KEY_V1: &[u8] = b"starcoin::metadata_v1";

/// Minimal compatible version of `RuntimeModuleMetadataV1` for deserialization.
/// Field order and types must match `starcoin-framework::module_metadata`.
#[derive(serde::Deserialize)]
struct RuntimeModuleMetadataV1 {
    #[allow(dead_code)]
    error_map: BTreeMap<u64, ErrorDescription>,
    #[allow(dead_code)]
    struct_attributes: BTreeMap<String, Vec<KnownAttribute>>,
    fun_attributes: BTreeMap<String, Vec<KnownAttribute>>,
}

/// Must match `move_core_types::errmap::ErrorDescription` for BCS compatibility.
#[derive(serde::Deserialize)]
struct ErrorDescription {
    #[allow(dead_code)]
    code_name: String,
    #[allow(dead_code)]
    code_description: String,
}

/// Minimal compatible version of `KnownAttribute` for deserialization.
#[derive(serde::Deserialize)]
struct KnownAttribute {
    kind: u8,
    #[allow(dead_code)]
    args: Vec<String>,
}

impl KnownAttribute {
    /// Returns `true` if this attribute is a `#[view]` annotation
    /// (supports both `LegacyViewFunction = 0` and `ViewFunction = 1`).
    fn is_view_function(&self) -> bool {
        self.kind == 0 || self.kind == 1
    }
}

/// Extract runtime module metadata from a compiled module's metadata section.
///
/// Tries V1 (with function attributes) first, then falls back to V0 (no
/// attributes — the fun_attributes map will be empty).
///
/// Returns:
/// - parsed metadata when available and valid
/// - `None` when metadata is unavailable
/// - an error message when metadata exists but cannot be decoded
fn get_metadata_from_compiled_module(
    module: &CompiledModule,
) -> (Option<RuntimeModuleMetadataV1>, Option<String>) {
    if let Some(data) = module
        .metadata
        .iter()
        .find(|md| md.key == STARCOIN_METADATA_KEY_V1)
    {
        match bcs::from_bytes::<RuntimeModuleMetadataV1>(&data.value) {
            Ok(metadata) => return (Some(metadata), None),
            Err(err) => {
                return (None, Some(format!("runtime metadata decode failed: {err}")));
            }
        }
    }
    // V0 has no attribute data. Return None so callers know metadata is
    // unavailable and is_view defaults to false.
    (None, None)
}

// ---------------------------------------------------------------------------
// Semantic resolver – thin wrapper over existing VM2 resolver
// ---------------------------------------------------------------------------

/// Derives [`ModuleSemantics`] from on-chain compiled Move modules.
///
/// This resolver reads compiled bytecode and existing ABI data but does **not**
/// execute or simulate transactions.  Effect hints are limited to what can be
/// inferred statically; anything beyond that is reported as a limitation.
pub struct SemanticsResolver<'a> {
    resolver: Resolver<'a>,
}

impl<'a> SemanticsResolver<'a> {
    pub fn new(state: &'a dyn StateView) -> Self {
        Self {
            resolver: Resolver::new(state),
        }
    }

    pub fn new_with_module_cache(state: &'a dyn StateView, cache: ModuleCache) -> Self {
        Self {
            resolver: Resolver::new_with_cache(state, cache),
        }
    }

    // ------------------------------------------------------------------
    // Public API
    // ------------------------------------------------------------------

    /// Resolve the on-chain module identified by `module_id` into AI-readable
    /// [`ModuleSemantics`].
    pub fn resolve_module(&self, module_id: &ModuleId) -> Result<ModuleSemantics> {
        let module = self
            .resolver
            .get_module(module_id.address(), module_id.name())?;
        self.resolve_compiled_module(module.as_ref())
    }

    /// Resolve a raw compiled module blob (not necessarily on-chain) into
    /// [`ModuleSemantics`].
    pub fn resolve_module_code(&self, code: &[u8]) -> Result<ModuleSemantics> {
        let module = CompiledModule::deserialize(code)?;
        self.resolver.update_cache(module.clone());
        self.resolve_compiled_module(&module)
    }

    // ------------------------------------------------------------------
    // Resolution helpers
    // ------------------------------------------------------------------

    fn resolve_compiled_module(&self, module: &CompiledModule) -> Result<ModuleSemantics> {
        let normalized = Module::new(module);
        let module_id = normalized.module_id();
        let metadata = runtime_module_metadata(module);

        let mut function_defs = module
            .function_defs()
            .iter()
            .enumerate()
            .map(|(idx, def)| {
                let handle = module.function_handle_at(def.function);
                let name = module.identifier_at(handle.name).to_owned();
                (name, idx)
            })
            .collect::<Vec<(Identifier, usize)>>();
        function_defs.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

        // Compute hashes
        let bytecode_bytes = {
            let mut buf = vec![];
            module.serialize(&mut buf)?;
            buf
        };
        let bytecode_hash = sha3_256_hex(&bytecode_bytes);

        // Interface hash covers: module id + all function signatures +
        // struct signatures + abilities, with deterministic ordering.
        let interface_hash =
            self.compute_interface_hash(module, &normalized, &function_defs, metadata.as_ref())?;

        // Collect ALL function definitions (public, entry, friend, private).
        let functions = function_defs
            .iter()
            .map(|(_name, idx)| {
                let def = &module.function_defs()[*idx];
                let (function_name, func) = Function::new(module, def);
                let is_view = is_view_function(metadata.as_ref(), function_name.as_ident_str());
                self.function_to_semantics(function_name.as_ident_str(), &func, is_view)
            })
            .collect::<Result<Vec<_>>>()?;

        let structs = normalized
            .structs
            .iter()
            .map(|(name, s)| struct_to_semantics(name.as_ident_str(), s))
            .collect();

        let limitations = vec![
            "Effect hints are bytecode-derived only; dry-run preview effects are not included"
                .to_string(),
            "Interface hash covers all functions (public/friend/private), function signatures, and struct signatures".to_string(),
        ];
        let limitations = limitations;

        Ok(ModuleSemantics {
            module: module_id.to_string(),
            bytecode_hash,
            interface_hash,
            functions,
            structs,
            runtime_attributes: Vec::new(),
            limitations,
        })
    }

    // ------------------------------------------------------------------
    // Function semantics
    // ------------------------------------------------------------------

    fn function_to_semantics(
        &self,
        name: &IdentStr,
        func: &Function,
        is_view: bool,
    ) -> Result<FunctionSemantics> {
        let visibility = func.visibility;
        let is_entry = func.is_entry;
        let type_parameters: Vec<String> = func
            .type_parameters
            .iter()
            .enumerate()
            .map(|(i, _)| format!("T{}", i))
            .collect();

        let parameters: Vec<String> = func.parameters.iter().map(|t| format!("{}", t)).collect();

        let returns: Vec<String> = func.return_.iter().map(|t| format!("{}", t)).collect();

        // Conservatively derive effect hints from visibility + entry status.
        let mut effect_hints = Vec::new();
        if is_entry {
            effect_hints.push(EffectHint::new("writes_resources", Provenance::Bytecode));
        } else if visibility == Visibility::Public {
            effect_hints.push(EffectHint::new("may_write_resources", Provenance::Bytecode));
        }

        Ok(FunctionSemantics {
            name: name.to_string(),
            visibility: visibility_to_str(visibility).to_string(),
            is_entry,
            is_view,
            type_parameters,
            parameters,
            returns,
            effect_hints,
            attributes: Vec::new(),
        })
    }

    // ------------------------------------------------------------------
    // Interface hash
    // ------------------------------------------------------------------

    /// Compute a deterministic hash of the module's interface surface:
    /// functions (name + visibility + entry + parameter types + return types)
    /// and structs (name + abilities + field types), plus the module ID.
    ///
    /// This hash is stable across bytecode changes that do not alter the
    /// exposed interface.
    fn compute_interface_hash(
        &self,
        module: &CompiledModule,
        normalized: &Module,
        function_defs: &[(Identifier, usize)],
        metadata: Option<&RuntimeMetadata>,
    ) -> Result<String> {
        use std::fmt::Write;

        let mut surface = String::new();
        writeln!(surface, "module:{}", normalized.module_id())?;

        // Functions – deterministic order.
        for (name, idx) in function_defs {
            let def = &module.function_defs()[*idx];
            let func = Function::new(module, def).1;
            let vis = visibility_to_str(def.visibility);
            let is_view = is_view_function(metadata, name.as_ident_str());

            write!(
                surface,
                "fn:{} vis:{} entry:{} view:{}",
                name, vis, func.is_entry, is_view
            )?;
            for (i, _) in func.type_parameters.iter().enumerate() {
                write!(surface, " tparam:T{}", i)?;
            }
            for p in &func.parameters {
                write!(surface, " param:{}", p)?;
            }
            for r in &func.return_ {
                write!(surface, " ret:{}", r)?;
            }
            writeln!(surface)?;
        }

        // Structs – deterministic order.
        let mut struct_names: Vec<&Identifier> = normalized.structs.keys().collect();
        struct_names.sort();

        for name in struct_names {
            let s = &normalized.structs[name];
            write!(surface, "struct:{}", name)?;
            let mut ability_strs: Vec<String> =
                s.abilities.into_iter().map(|a| format!("{}", a)).collect();
            ability_strs.sort();
            for a in &ability_strs {
                write!(surface, " abil:{}", a)?;
            }
            for (i, _) in s.type_parameters.iter().enumerate() {
                write!(surface, " tparam:T{}", i)?;
            }
            for f in &s.fields {
                write!(surface, " field:{}:{}", f.name, f.type_)?;
            }
            writeln!(surface)?;
        }

        Ok(sha3_256_hex(surface.as_bytes()))
    }
}

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------

fn ability_names(abilities: AbilitySet) -> Vec<String> {
    abilities
        .into_iter()
        .map(|ability| ability.to_string())
        .collect()
}

fn struct_to_semantics(name: &IdentStr, struct_: &Struct) -> StructSemantics {
    let type_parameters = struct_
        .type_parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            StructTypeParameterSemantics::new(
                format!("T{}", index),
                ability_names(parameter.constraints),
                parameter.is_phantom,
                Provenance::Bytecode,
            )
        })
        .collect();

    let fields = struct_
        .fields
        .iter()
        .map(|field| {
            FieldSemantics::new(
                field.name.to_string(),
                field.type_.to_string(),
                Provenance::Bytecode,
            )
        })
        .collect();

    StructSemantics {
        name: name.to_string(),
        abilities: ability_names(struct_.abilities),
        abilities_source: Provenance::Bytecode,
        is_resource: struct_.abilities.has_key(),
        is_resource_source: Provenance::Bytecode,
        type_parameters,
        fields,
        attributes: Vec::new(),
    }
}

fn visibility_to_str(v: Visibility) -> &'static str {
    match v {
        Visibility::Public => "public",
        Visibility::Friend => "friend",
        Visibility::Private => "private",
    }
}

#[cfg(feature = "ai-metadata")]
const STARCOIN_METADATA_KEY: &[u8] = b"starcoin::metadata_v0";
#[cfg(feature = "ai-metadata")]
const STARCOIN_METADATA_KEY_V1: &[u8] = b"starcoin::metadata_v1";

#[cfg(not(feature = "ai-metadata"))]
type RuntimeMetadata = ();

#[cfg(feature = "ai-metadata")]
type RuntimeMetadata = RuntimeModuleMetadataV1;

#[cfg(feature = "ai-metadata")]
fn runtime_module_metadata(module: &CompiledModule) -> Option<RuntimeMetadata> {
    module.metadata.iter().find_map(|metadata| {
        if metadata.key == STARCOIN_METADATA_KEY {
            Some(RuntimeModuleMetadataV1::default())
        } else if metadata.key == STARCOIN_METADATA_KEY_V1 {
            bcs::from_bytes::<RuntimeModuleMetadataV1>(&metadata.value).ok()
        } else {
            None
        }
    })
}

#[cfg(not(feature = "ai-metadata"))]
fn runtime_module_metadata(_module: &CompiledModule) -> Option<RuntimeMetadata> {
    None
}

#[cfg(feature = "ai-metadata")]
#[derive(serde::Deserialize)]
#[cfg(feature = "ai-metadata")]
struct RuntimeModuleMetadataV1 {
    #[serde(default)]
    fun_attributes: std::collections::BTreeMap<String, Vec<KnownAttribute>>,
}

#[cfg(feature = "ai-metadata")]
impl RuntimeModuleMetadataV1 {
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        Self {
            fun_attributes: std::collections::BTreeMap::new(),
        }
    }
}

#[derive(serde::Deserialize)]
#[cfg(feature = "ai-metadata")]
struct KnownAttribute {
    kind: u8,
    #[allow(dead_code)]
    #[serde(default)]
    args: Vec<String>,
}

#[cfg(feature = "ai-metadata")]
impl KnownAttribute {
    fn is_view_function(&self) -> bool {
        self.kind == 0 || self.kind == 1
    }
}

#[cfg(feature = "ai-metadata")]
fn is_view_function(metadata: Option<&RuntimeMetadata>, name: &IdentStr) -> bool {
    metadata
        .and_then(|metadata| metadata.fun_attributes.get(name.as_str()))
        .map(|attrs| attrs.iter().any(|attr| attr.is_view_function()))
        .unwrap_or(false)
}

#[cfg(not(feature = "ai-metadata"))]
fn is_view_function(_metadata: Option<&RuntimeMetadata>, _name: &IdentStr) -> bool {
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_vm2_vm_types::access_path::DataPath;
    use starcoin_vm2_vm_types::account_config::genesis_address;
    use starcoin_vm2_vm_types::file_format::{
        Ability, AbilitySet, CompiledModule, StructTypeParameter,
    };
    use starcoin_vm2_vm_types::identifier::Identifier;
    use starcoin_vm2_vm_types::language_storage::ModuleId;
    use starcoin_vm2_vm_types::normalized::{Field, Type};
    use starcoin_vm2_vm_types::state_store::errors::StateviewError;
    use starcoin_vm2_vm_types::state_store::state_key::inner::StateKeyInner;
    use starcoin_vm2_vm_types::state_store::state_key::StateKey;
    use starcoin_vm2_vm_types::state_store::state_storage_usage::StateStorageUsage;
    use starcoin_vm2_vm_types::state_store::state_value::StateValue;
    use starcoin_vm2_vm_types::state_store::TStateView;
    use std::collections::BTreeMap;

    // -- In-memory state view (same pattern as the ABI resolver tests) ------

    pub struct InMemoryStateView {
        modules: BTreeMap<ModuleId, CompiledModule>,
    }

    impl InMemoryStateView {
        pub fn new(modules: Vec<CompiledModule>) -> Self {
            Self {
                modules: modules.into_iter().map(|m| (m.self_id(), m)).collect(),
            }
        }
    }

    impl TStateView for InMemoryStateView {
        type Key = StateKey;

        fn get_state_value(
            &self,
            state_key: &StateKey,
        ) -> std::result::Result<Option<StateValue>, StateviewError> {
            match state_key.inner() {
                StateKeyInner::AccessPath(access_path) => {
                    let module_id = match &access_path.path {
                        DataPath::Code(name) => ModuleId::new(access_path.address, name.clone()),
                        _ => return Err(StateviewError::Other("no data".to_string())),
                    };
                    Ok(self
                        .modules
                        .get(&module_id)
                        .map(|m| {
                            let mut data = vec![];
                            m.serialize(&mut data).unwrap();
                            data
                        })
                        .map(StateValue::from))
                }
                _ => Err(StateviewError::Other("unexpected key type".to_string())),
            }
        }

        fn get_usage(&self) -> starcoin_vm2_vm_types::state_store::Result<StateStorageUsage> {
            todo!()
        }

        fn is_genesis(&self) -> bool {
            todo!()
        }
    }

    // -- Tests -------------------------------------------------------------

    #[test]
    fn test_resolve_dao_module_semantics() {
        let modules = starcoin_cached_packages::head_release_bundle().compiled_modules();
        let view = InMemoryStateView::new(modules);
        let resolver = SemanticsResolver::new(&view);

        let module_id = ModuleId::new(genesis_address(), Identifier::new("dao").unwrap());
        let semantics = resolver.resolve_module(&module_id).unwrap();

        // Module identity
        assert_eq!(semantics.module, "0x00000000000000000000000000000001::dao");
        assert!(semantics.bytecode_hash.starts_with("0x"));
        assert!(semantics.interface_hash.starts_with("0x"));
        assert_eq!(semantics.bytecode_hash.len(), 66);
        assert_eq!(semantics.interface_hash.len(), 66);

        // Functions — verify keys exist with correct structure.
        assert!(!semantics.functions.is_empty());
        let cast_vote = semantics
            .functions
            .iter()
            .find(|f| f.name == "cast_vote")
            .expect("cast_vote not found");
        assert_eq!(cast_vote.visibility, "public");
        // cast_vote is a public function; entry-ness depends on framework version.
        assert!(!cast_vote.is_view);

        // There should be at least one entry function in the dao module.
        let entry_fns: Vec<_> = semantics.functions.iter().filter(|f| f.is_entry).collect();
        assert!(!entry_fns.is_empty(), "dao module has no entry functions");

        // Structs
        assert!(!semantics.structs.is_empty());
        let proposal = semantics
            .structs
            .iter()
            .find(|s| s.name == "Proposal")
            .expect("Proposal not found");
        assert!(proposal.is_resource);
        assert_eq!(proposal.abilities, vec!["key"]);
        assert_eq!(proposal.abilities_source, Provenance::Bytecode);
        assert_eq!(proposal.is_resource_source, Provenance::Bytecode);
        assert_eq!(proposal.type_parameters.len(), 2);
        assert_eq!(proposal.type_parameters[0].name, "T0");
        assert!(proposal.type_parameters[0].constraints.is_empty());
        assert!(proposal.type_parameters[0].is_phantom);
        assert_eq!(proposal.type_parameters[1].name, "T1");
        assert_eq!(proposal.type_parameters[1].constraints, vec!["store"]);
        assert!(!proposal.type_parameters[1].is_phantom);
        let action = proposal
            .fields
            .iter()
            .find(|field| field.name == "action")
            .expect("Proposal::action not found");
        assert_eq!(action.type_, "0x1::option::Option<T1>");
        assert_eq!(action.source, Provenance::Bytecode);

        // Limitations reported
        assert!(!semantics.limitations.is_empty());

        // JSON round-trip
        let json = serde_json::to_string_pretty(&semantics).unwrap();
        let back: ModuleSemantics = serde_json::from_str(&json).unwrap();
        assert_eq!(semantics, back);
    }

    #[test]
    fn test_deterministic_hashes() {
        let modules = starcoin_cached_packages::head_release_bundle().compiled_modules();
        let view = InMemoryStateView::new(modules);
        let resolver = SemanticsResolver::new(&view);

        let module_id = ModuleId::new(genesis_address(), Identifier::new("dao").unwrap());
        let s1 = resolver.resolve_module(&module_id).unwrap();
        let s2 = resolver.resolve_module(&module_id).unwrap();

        // Same module → same hashes
        assert_eq!(s1.bytecode_hash, s2.bytecode_hash);
        assert_eq!(s1.interface_hash, s2.interface_hash);
    }

    #[test]
    fn test_struct_semantics_preserves_all_ability_combinations() {
        let name = Identifier::new("AbilityCarrier").unwrap();

        for byte in 0..=AbilitySet::ALL.into_u8() {
            let abilities = AbilitySet::from_u8(byte).expect("valid ability combination");
            let struct_ = Struct {
                abilities,
                type_parameters: vec![],
                fields: vec![],
            };
            let semantics = struct_to_semantics(name.as_ident_str(), &struct_);

            let expected_abilities = [
                (Ability::Copy, "copy"),
                (Ability::Drop, "drop"),
                (Ability::Store, "store"),
                (Ability::Key, "key"),
            ]
            .into_iter()
            .filter_map(|(ability, name)| abilities.has_ability(ability).then_some(name))
            .collect::<Vec<_>>();

            assert_eq!(
                semantics.abilities, expected_abilities,
                "ability byte {byte:#x}"
            );
            assert_eq!(
                semantics.is_resource,
                abilities.has_key(),
                "ability byte {byte:#x}"
            );
        }
    }

    #[test]
    fn test_struct_semantics_preserves_generics_and_nested_types() {
        let struct_ = Struct {
            abilities: AbilitySet::EMPTY | Ability::Key,
            type_parameters: vec![
                StructTypeParameter {
                    constraints: AbilitySet::EMPTY,
                    is_phantom: true,
                },
                StructTypeParameter {
                    constraints: AbilitySet::EMPTY | Ability::Copy | Ability::Drop | Ability::Store,
                    is_phantom: false,
                },
            ],
            fields: vec![Field {
                name: Identifier::new("nested").unwrap(),
                type_: Type::Struct {
                    address: genesis_address(),
                    module: Identifier::new("option").unwrap(),
                    name: Identifier::new("Option").unwrap(),
                    type_arguments: vec![Type::TypeParameter(1)],
                },
            }],
        };
        let name = Identifier::new("GenericResource").unwrap();

        let semantics = struct_to_semantics(name.as_ident_str(), &struct_);

        assert_eq!(
            semantics.type_parameters,
            vec![
                StructTypeParameterSemantics::new("T0", vec![], true, Provenance::Bytecode,),
                StructTypeParameterSemantics::new(
                    "T1",
                    vec!["copy".into(), "drop".into(), "store".into()],
                    false,
                    Provenance::Bytecode,
                ),
            ]
        );
        assert_eq!(
            semantics.fields,
            vec![FieldSemantics::new(
                "nested",
                "0x1::option::Option<T1>",
                Provenance::Bytecode,
            )]
        );
    }

    #[test]
    fn test_view_function_detection() {
        let modules = starcoin_cached_packages::head_release_bundle().compiled_modules();
        let view = InMemoryStateView::new(modules);
        let resolver = SemanticsResolver::new(&view);

        // chain_status::is_genesis and is_operating are #[view] functions.
        let module_id = ModuleId::new(genesis_address(), Identifier::new("chain_status").unwrap());
        let semantics = resolver.resolve_module(&module_id).unwrap();

        let is_genesis = semantics
            .functions
            .iter()
            .find(|f| f.name == "is_genesis")
            .expect("is_genesis not found");
        assert!(is_genesis.is_view, "is_genesis should be a view function");

        let is_operating = semantics
            .functions
            .iter()
            .find(|f| f.name == "is_operating")
            .expect("is_operating not found");
        assert!(
            is_operating.is_view,
            "is_operating should be a view function"
        );

        // assert_operating is NOT a view function.
        let assert_operating = semantics
            .functions
            .iter()
            .find(|f| f.name == "assert_operating")
            .expect("assert_operating not found");
        assert!(
            !assert_operating.is_view,
            "assert_operating should NOT be a view function"
        );
    }

    #[test]
    fn test_invalid_runtime_metadata_is_reported_in_limitations() {
        let modules = starcoin_cached_packages::head_release_bundle().compiled_modules();
        let dao = modules
            .iter()
            .find(|m| {
                m.self_id() == ModuleId::new(genesis_address(), Identifier::new("dao").unwrap())
            })
            .expect("dao module not found in framework")
            .clone();

        let mut dao = dao;
        dao.metadata.retain(|md| md.key != STARCOIN_METADATA_KEY_V1);
        dao.metadata.push(Metadata {
            key: STARCOIN_METADATA_KEY_V1.to_vec(),
            value: vec![1, 2, 3, 4],
        });

        let mut code_bytes = vec![];
        dao.serialize(&mut code_bytes).unwrap();

        let view = InMemoryStateView::new(modules);
        let resolver = SemanticsResolver::new(&view);
        let semantics = resolver.resolve_module_code(&code_bytes).unwrap();

        assert!(
            semantics
                .limitations
                .iter()
                .any(|item| item.contains("Runtime metadata decode failed")),
            "missing metadata decode limitation: {:?}",
            semantics.limitations
        );
    }

    #[test]
    fn test_resolve_module_code_from_bytes() {
        // Deserialize a known framework module and re-resolve it via raw code path.
        let modules = starcoin_cached_packages::head_release_bundle().compiled_modules();
        let dao = modules
            .iter()
            .find(|m| {
                m.self_id() == ModuleId::new(genesis_address(), Identifier::new("dao").unwrap())
            })
            .expect("dao module not found in framework");

        let mut code_bytes = vec![];
        dao.serialize(&mut code_bytes).unwrap();

        let view = InMemoryStateView::new(modules);
        let resolver = SemanticsResolver::new(&view);
        let semantics = resolver.resolve_module_code(&code_bytes).unwrap();

        assert_eq!(semantics.module, "0x00000000000000000000000000000001::dao");
        assert!(semantics.bytecode_hash.starts_with("0x"));
        assert!(!semantics.functions.is_empty());
        assert!(!semantics.structs.is_empty());
    }
}
