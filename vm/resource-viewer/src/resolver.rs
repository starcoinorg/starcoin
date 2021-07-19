// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    fat_type::{FatStructType, FatType, WrappedAbilitySet},
    module_cache::ModuleCache,
};
use anyhow::{anyhow, Result};
use starcoin_vm_types::{
    access::ModuleAccess,
    access_path::AccessPath,
    account_address::AccountAddress,
    errors::PartialVMError,
    file_format::{
        CompiledModule, SignatureToken, StructDefinitionIndex, StructFieldInformation,
        StructHandleIndex,
    },
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
    state_view::StateView,
};
use std::rc::Rc;

pub struct Resolver<'a> {
    state: &'a dyn StateView,
    cache: ModuleCache,
}

impl<'a> Resolver<'a> {
    pub fn new(state: &'a dyn StateView) -> Self {
        let cache = ModuleCache::new();
        Self::new_with_cache(state, cache)
    }

    pub fn new_with_cache(state: &'a dyn StateView, cache: ModuleCache) -> Self {
        Resolver { state, cache }
    }

    pub fn update_cache(&self, module: CompiledModule) {
        self.cache.insert(module.self_id(), module);
    }

    pub fn get_module(
        &self,
        address: &AccountAddress,
        name: &IdentStr,
    ) -> Result<Rc<CompiledModule>> {
        let module_id = ModuleId::new(*address, name.to_owned());
        if let Some(module) = self.cache.get(&module_id) {
            return Ok(module);
        }
        let access_path = AccessPath::from(&module_id);
        let blob = self
            .state
            .get(&access_path)?
            .ok_or_else(|| anyhow!("Module {:?} can't be found", module_id))?;
        let compiled_module = CompiledModule::deserialize(&blob).map_err(|status| {
            anyhow!(
                "Module {:?} deserialize with error code {:?}",
                module_id,
                status
            )
        })?;
        Ok(self.cache.insert(module_id, compiled_module))
    }

    pub(crate) fn resolve_type(&self, type_tag: &TypeTag) -> Result<FatType> {
        Ok(match type_tag {
            TypeTag::Address => FatType::Address,
            TypeTag::Signer => FatType::Signer,
            TypeTag::Bool => FatType::Bool,
            TypeTag::Struct(st) => FatType::Struct(Box::new(self.resolve_struct(st)?)),
            TypeTag::U8 => FatType::U8,
            TypeTag::U64 => FatType::U64,
            TypeTag::U128 => FatType::U128,
            TypeTag::Vector(ty) => FatType::Vector(Box::new(self.resolve_type(ty)?)),
        })
    }

    pub(crate) fn resolve_struct(&self, struct_tag: &StructTag) -> Result<FatStructType> {
        let module = self.get_module(&struct_tag.address, &struct_tag.module)?;
        let struct_def =
            find_struct_def_in_module(module.as_ref(), struct_tag.name.as_ident_str())?;
        let ty_args = struct_tag
            .type_params
            .iter()
            .map(|ty| self.resolve_type(ty))
            .collect::<Result<Vec<_>>>()?;
        let ty_body = self.resolve_struct_definition(module.as_ref(), struct_def)?;
        ty_body.subst(&ty_args).map_err(|e: PartialVMError| {
            anyhow!("StructTag {:?} cannot be resolved: {:?}", struct_tag, e)
        })
    }

    pub(crate) fn get_field_names(&self, ty: &FatStructType) -> Result<Vec<Identifier>> {
        let module = self.get_module(&ty.address, ty.module.as_ident_str())?;
        let struct_def_idx = find_struct_def_in_module(module.as_ref(), ty.name.as_ident_str())?;
        let struct_def = module.struct_def_at(struct_def_idx);

        match &struct_def.field_information {
            StructFieldInformation::Native => Err(anyhow!("Unexpected Native Struct")),
            StructFieldInformation::Declared(defs) => Ok(defs
                .iter()
                .map(|field_def| module.identifier_at(field_def.name).to_owned())
                .collect()),
        }
    }

    pub(crate) fn resolve_signature(
        &self,
        module: &CompiledModule,
        sig: &SignatureToken,
    ) -> Result<FatType> {
        Ok(match sig {
            SignatureToken::Bool => FatType::Bool,
            SignatureToken::U8 => FatType::U8,
            SignatureToken::U64 => FatType::U64,
            SignatureToken::U128 => FatType::U128,
            SignatureToken::Address => FatType::Address,
            SignatureToken::Signer => FatType::Signer,
            SignatureToken::Vector(ty) => {
                FatType::Vector(Box::new(self.resolve_signature(module, ty)?))
            }
            SignatureToken::Struct(idx) => {
                FatType::Struct(Box::new(self.resolve_struct_handle(module, *idx)?))
            }
            SignatureToken::StructInstantiation(idx, toks) => {
                let struct_ty = self.resolve_struct_handle(module, *idx)?;
                let args = toks
                    .iter()
                    .map(|tok| self.resolve_signature(module, tok))
                    .collect::<Result<Vec<_>>>()?;
                FatType::Struct(Box::new(
                    struct_ty
                        .subst(&args)
                        .map_err(|status| anyhow!("Substitution failure: {:?}", status))?,
                ))
            }
            SignatureToken::TypeParameter(idx) => FatType::TyParam(*idx as usize),
            SignatureToken::MutableReference(_) | SignatureToken::Reference(_) => {
                return Err(anyhow!("Unexpected Reference"))
            }
        })
    }

    pub(crate) fn resolve_struct_handle(
        &self,
        module: &CompiledModule,
        idx: StructHandleIndex,
    ) -> Result<FatStructType> {
        let struct_handle = module.struct_handle_at(idx);
        let target_module = {
            let module_handle = module.module_handle_at(struct_handle.module);
            self.get_module(
                module.address_identifier_at(module_handle.address),
                module.identifier_at(module_handle.name),
            )?
        };
        let target_idx = find_struct_def_in_module(
            target_module.as_ref(),
            module.identifier_at(struct_handle.name),
        )?;
        self.resolve_struct_definition(target_module.as_ref(), target_idx)
    }

    pub(crate) fn resolve_struct_definition(
        &self,
        module: &CompiledModule,
        idx: StructDefinitionIndex,
    ) -> Result<FatStructType> {
        let struct_def = module.struct_def_at(idx);
        let struct_handle = module.struct_handle_at(struct_def.struct_handle);
        let address = *module.address();
        let module_name = module.name().to_owned();
        let name = module.identifier_at(struct_handle.name).to_owned();
        let abilities = struct_handle.abilities;
        let ty_args = (0..struct_handle.type_parameters.len())
            .map(FatType::TyParam)
            .collect();
        match &struct_def.field_information {
            StructFieldInformation::Native => Err(anyhow!("Unexpected Native Struct")),
            StructFieldInformation::Declared(defs) => Ok(FatStructType {
                address,
                module: module_name,
                name,
                abilities: WrappedAbilitySet(abilities),
                ty_args,
                layout: defs
                    .iter()
                    .map(|field_def| self.resolve_signature(module, &field_def.signature.0))
                    .collect::<Result<_>>()?,
            }),
        }
    }
}

fn find_struct_def_in_module(
    module: &CompiledModule,
    name: &IdentStr,
) -> Result<StructDefinitionIndex> {
    for (i, defs) in module.struct_defs().iter().enumerate() {
        let st_handle = module.struct_handle_at(defs.struct_handle);
        if module.identifier_at(st_handle.name) == name {
            return Ok(StructDefinitionIndex::new(i as u16));
        }
    }
    Err(anyhow!(
        "Struct {:?} not found in {:?}",
        name,
        module.self_id()
    ))
}
