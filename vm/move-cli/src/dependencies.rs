use anyhow::Result;
use itertools::Itertools;
use move_core_types::language_storage::ModuleId;
use move_lang::parser::ast::{
    Definition, LeadingNameAccess_, ModuleDefinition, ModuleIdent, ModuleMember, Use,
};
use move_lang::shared::{AddressBytes as Address, CompilationEnv, Flags, Identifier};
use move_vm_runtime::data_cache::MoveStorage;
use std::collections::{btree_map, BTreeMap};
use vm::access::ModuleAccess;
use vm::CompiledModule;

pub trait ModuleDependencyResolver: MoveStorage + Sized {
    fn get_module_dependencies_recursively(
        &self,
        module: &CompiledModule,
    ) -> Result<BTreeMap<ModuleId, CompiledModule>> {
        let mut all_deps = BTreeMap::new();
        for dep in module.immediate_dependencies() {
            get_all_module_dependencies_recursive(&mut all_deps, dep, self)?;
        }
        Ok(all_deps)
    }
    fn get_module_dependencies_recursively_for_all(
        &self,
        modules: &[CompiledModule],
    ) -> Result<BTreeMap<ModuleId, CompiledModule>> {
        let mut all_deps = BTreeMap::new();
        for dep in modules
            .iter()
            .flat_map(|m| m.immediate_dependencies())
            .unique()
        {
            get_all_module_dependencies_recursive(&mut all_deps, dep, self)?;
        }
        Ok(all_deps)
    }
}

fn get_all_module_dependencies_recursive<R: MoveStorage + Sized>(
    all_deps: &mut BTreeMap<ModuleId, CompiledModule>,
    module_id: ModuleId,
    loader: &R,
) -> Result<()> {
    if let btree_map::Entry::Vacant(entry) = all_deps.entry(module_id) {
        let module = loader
            .get_module(entry.key())
            .map_err(|e| e.into_vm_status())?
            .ok_or_else(|| anyhow::anyhow!("missing dependency {:?}", entry.key()))?;
        let module = CompiledModule::deserialize(&module)?;
        let next_deps = module.immediate_dependencies();
        entry.insert(module);
        for next in next_deps {
            get_all_module_dependencies_recursive(all_deps, next, loader)?;
        }
    }
    Ok(())
}

impl<R> ModuleDependencyResolver for R where R: MoveStorage + Sized {}

pub fn get_uses(move_files: &[String]) -> Result<Vec<(Address, String)>> {
    fn get_module_uses(m: &ModuleDefinition) -> Vec<ModuleIdent> {
        m.members
            .iter()
            .filter_map(|m| {
                if let ModuleMember::Use(u) = m {
                    Some(match &u.use_ {
                        Use::Module(mi, _) => mi.clone(),
                        Use::Members(mi, _) => mi.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    let mut compilation_env = CompilationEnv::new(Flags::empty());
    let (files, parsed) = move_lang::move_parse(&compilation_env, move_files, &[], None)?;

    let (_, program) = move_lang::unwrap_or_report_errors!(files, parsed);

    let address_mapping =
        move_lang::expansion::address_map::build_address_map(&mut compilation_env, None, &program);
    let expansion_errors = compilation_env.check_errors();
    move_lang::unwrap_or_report_errors!(files, expansion_errors);

    let used_deps: Vec<_> = program
        .source_definitions
        .into_iter()
        .flat_map(|d| match d {
            Definition::Module(m) => get_module_uses(&m),
            Definition::Address(ad) => ad.modules.iter().flat_map(|m| get_module_uses(m)).collect(),
            Definition::Script(s) => s
                .uses
                .iter()
                .map(|u| match &u.use_ {
                    Use::Module(mi, _) => mi.clone(),
                    Use::Members(mi, _) => mi.clone(),
                })
                .collect(),
        })
        .unique()
        .collect();

    let mapped_used_deps = used_deps
        .into_iter()
        .filter_map(|elem| {
            let elem = elem.value;
            let module_name = elem.module.value().to_string();
            let addr = match elem.address.value {
                LeadingNameAccess_::AnonymousAddress(addr) => Some(addr),
                LeadingNameAccess_::Name(addr_name) => address_mapping
                    .get(&addr_name)
                    .and_then(|a| a.clone().map(|b| b.value)),
            };
            addr.map(|a| (a, module_name))
        })
        .unique()
        .collect();
    Ok(mapped_used_deps)
}
