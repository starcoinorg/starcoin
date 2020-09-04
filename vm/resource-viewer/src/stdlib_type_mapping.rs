use crate::module_cache::ModuleCache;
use crate::resolver::Resolver;
use crate::NullStateView;
use anyhow::{bail, Result};
use once_cell::sync::Lazy;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::file_format::StructDefinitionIndex;
use starcoin_vm_types::language_storage::StructTag;
use std::collections::BTreeMap;
use stdlib::{stdlib_modules, StdLibOptions};

#[allow(unused)]
pub static COMPILED_TYPE_MAP: Lazy<BTreeMap<Vec<u8>, StructTag>> =
    Lazy::new(|| generate_stdlib_type_mapping().unwrap());

/// NOTICE: this does not support generic struct type.
fn generate_stdlib_type_mapping() -> Result<BTreeMap<Vec<u8>, StructTag>> {
    let compiled_modules = stdlib_modules(StdLibOptions::Staged);
    let cache = ModuleCache::new();
    for m in compiled_modules {
        cache.insert(m.self_id(), m.clone());
    }
    let state_view = NullStateView::default();
    let resolver = Resolver::new_with_cache(&state_view, cache);

    let mut type_mappings = BTreeMap::new();

    for m in compiled_modules {
        for (i, _) in m.struct_defs().iter().enumerate() {
            let struct_type =
                resolver.resolve_struct_definition(m, StructDefinitionIndex(i as u16))?;

            // skip non-resource struct
            if !struct_type.is_resource {
                continue;
            }

            // let ty_parameters = m.signature_at(inst.type_parameters).0.clone();
            //
            // let mut instantiated_ty_parameters = vec![];
            // for t in ty_parameters {
            //     let ty = resolver.resolve_signature(m, &t)?;
            //     instantiated_ty_parameters.push(ty);
            // }
            // struct_type.ty_args = instantiated_ty_parameters;

            // only add no generic types
            if struct_type.ty_args.is_empty() {
                match struct_type.struct_tag() {
                    Err(e) => {
                        bail!(
                                "Module: {:?}, FatStructType {:?} cannot be converted to StructTag: {:?}",
                                m.self_id(),
                                &struct_type,
                                e
                            );
                    }
                    Ok(struct_tag) => {
                        type_mappings.insert(struct_tag.access_vector(), struct_tag);
                    }
                }
            }
        }
    }

    Ok(type_mappings)
}

#[test]
pub fn test_type_mapping() -> Result<()> {
    let mappings = generate_stdlib_type_mapping()?;
    assert!(!mappings.is_empty());
    for m in mappings {
        println!("{:?}", m.1);
    }
    Ok(())
}
