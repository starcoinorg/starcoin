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
use starcoin_vm2_abi_types::ai_semantics::{
    sha3_256_hex, EffectHint, FieldSemantics, FunctionSemantics, ModuleSemantics, Provenance,
    StructSemantics,
};
use starcoin_vm2_resource_viewer::module_cache::ModuleCache;
use starcoin_vm2_resource_viewer::resolver::Resolver;
use starcoin_vm2_vm_types::access::ModuleAccess;
use starcoin_vm2_vm_types::file_format::{
    CompiledModule, FunctionDefinitionIndex, Visibility,
};
use starcoin_vm2_vm_types::identifier::{IdentStr, Identifier};
use starcoin_vm2_vm_types::language_storage::ModuleId;
use starcoin_vm2_vm_types::normalized::{Function, Module, Struct};
use starcoin_vm2_vm_types::state_store::StateView;

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

        // Compute hashes
        let bytecode_bytes = {
            let mut buf = vec![];
            module.serialize(&mut buf)?;
            buf
        };
        let bytecode_hash = sha3_256_hex(&bytecode_bytes);

        // Interface hash covers: module id + callable function signatures +
        // struct signatures + abilities.  Excludes bytecode body and private
        // functions so it is stable across non-breaking bytecode changes.
        let interface_hash = self.compute_interface_hash(module, &normalized)?;

        // Collect ALL function definitions (public, entry, friend, private).
        let functions = module
            .function_defs()
            .iter()
            
            .map(|def| {
                let (name, func) = Function::new(module, def);
                self.function_to_semantics(name.as_ident_str(), &func)
            })
            .collect::<Result<Vec<_>>>()?;

        let structs = normalized
            .structs
            .iter()
            .map(|(name, s)| {
                let sname = name.as_ident_str();
                self.struct_to_semantics(sname, s)
            })
            .collect::<Result<Vec<_>>>()?;

        let limitations = vec![
            "Effect hints are bytecode-derived only; dry-run preview effects are not included"
                .to_string(),
            "View function detection is not yet implemented; is_view is always false".to_string(),
            "Interface hash covers only the public/entry callable surface; private/friend functions are excluded from the hash".to_string(),
        ];

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
    ) -> Result<FunctionSemantics> {
        let visibility = func.visibility;
        let is_entry = func.is_entry;

        // VM2 view functions are determined by the `#[view]` attribute
        // stored in bytecode metadata.  That metadata is not yet exposed
        // through the normalized `Function` struct, so we conservatively
        // report is_view=false for now.
        let is_view = false;

        let type_parameters: Vec<String> = func
            .type_parameters
            .iter()
            .enumerate()
            .map(|(i, _)| format!("T{}", i))
            .collect();

        let parameters: Vec<String> = func
            .parameters
            .iter()
            .map(|t| format!("{}", t))
            .collect();

        let returns: Vec<String> = func.return_.iter().map(|t| format!("{}", t)).collect();

        // Conservatively derive effect hints from visibility + entry status.
        let mut effect_hints = Vec::new();
        if is_entry {
            effect_hints.push(EffectHint::new(
                "writes_resources",
                Provenance::Bytecode,
            ));
        } else if visibility == Visibility::Public {
            effect_hints.push(EffectHint::new(
                "may_write_resources",
                Provenance::Bytecode,
            ));
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
    // Struct semantics
    // ------------------------------------------------------------------

    fn struct_to_semantics(
        &self,
        name: &IdentStr,
        s: &Struct,
    ) -> Result<StructSemantics> {
        let abilities: Vec<String> = s
            .abilities
            .into_iter()
            .map(|a| format!("{}", a))
            .collect();

        // A struct is a resource if it has the `key` ability.
        let is_resource = s.abilities.has_key();

        // Type parameter names (T0, T1, ...) — names from the struct handle
        // are not exposed through the normalized representation.
        let type_parameters: Vec<String> = s
            .type_parameters
            .iter()
            .enumerate()
            .map(|(i, _)| format!("T{}", i))
            .collect();

        let fields: Vec<FieldSemantics> = s
            .fields
            .iter()
            .map(|f| FieldSemantics::new(f.name.to_string(), format!("{}", f.type_)))
            .collect();

        Ok(StructSemantics {
            name: name.to_string(),
            abilities,
            is_resource,
            type_parameters,
            fields,
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
    ) -> Result<String> {
        use std::fmt::Write;

        let mut surface = String::new();
        writeln!(surface, "module:{}", normalized.module_id())?;

        // Functions – deterministic order.
        let mut func_names: Vec<&Identifier> =
            normalized.exposed_functions.keys().collect();
        func_names.sort();

        for name in func_names {
            let func = &normalized.exposed_functions[name];
            let def_idx = find_function_def_in_module(module, name.as_ident_str());
            let vis = def_idx
                .map(|idx| module.function_def_at(idx).visibility)
                .unwrap_or(Visibility::Private);

            write!(
                surface,
                "fn:{} vis:{} entry:{}",
                name,
                visibility_to_str(vis),
                func.is_entry
            )?;
            for p in &func.parameters {
                write!(surface, " param:{}", p)?;
            }
            for r in &func.return_ {
                write!(surface, " ret:{}", r)?;
            }
            writeln!(surface)?;
        }

        // Structs – deterministic order.
        let mut struct_names: Vec<&Identifier> =
            normalized.structs.keys().collect();
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

fn visibility_to_str(v: Visibility) -> &'static str {
    match v {
        Visibility::Public => "public",
        Visibility::Friend => "friend",
        Visibility::Private => "private",
    }
}

fn find_function_def_in_module(
    module: &CompiledModule,
    name: &IdentStr,
) -> Option<FunctionDefinitionIndex> {
    for (i, def) in module.function_defs().iter().enumerate() {
        let handle = module.function_handle_at(def.function);
        if module.identifier_at(handle.name) == name {
            return Some(FunctionDefinitionIndex::new(i as u16));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_vm2_vm_types::access_path::DataPath;
    use starcoin_vm2_vm_types::account_address::AccountAddress;
    use starcoin_vm2_vm_types::account_config::genesis_address;
    use starcoin_vm2_vm_types::file_format::CompiledModule;
    use starcoin_vm2_vm_types::identifier::Identifier;
    use starcoin_vm2_vm_types::language_storage::ModuleId;
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
                        DataPath::Code(name) => {
                            ModuleId::new(access_path.address, name.clone())
                        }
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

        // Functions
        assert!(!semantics.functions.is_empty());
        let cast_vote = semantics
            .functions
            .iter()
            .find(|f| f.name == "cast_vote")
            .expect("cast_vote not found");
        assert_eq!(cast_vote.visibility, "public");
        assert!(cast_vote.is_entry);
        assert!(!cast_vote.is_view);

        // Structs
        assert!(!semantics.structs.is_empty());
        let proposal = semantics
            .structs
            .iter()
            .find(|s| s.name == "Proposal")
            .expect("Proposal not found");
        assert!(proposal.is_resource);
        assert!(proposal.abilities.contains(&"key".to_string()));

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
    fn test_resolve_module_code_from_bytes() {
        // Deserialize a known framework module and re-resolve it via raw code path.
        let modules = starcoin_cached_packages::head_release_bundle().compiled_modules();
        let dao = modules
            .iter()
            .find(|m| {
                m.self_id()
                    == ModuleId::new(genesis_address(), Identifier::new("dao").unwrap())
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
