// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This mod define `Contract` for Module.
//! The meaning of `Contract` here is the same as in `Design by contract`.
//! It represents a commitment to a Module's external users.
//! For ensure compatibility, Module upgrade can not break the contract.

use anyhow::{anyhow, format_err, Result};
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::file_format::{
    CompiledModule, FieldDefinition, FunctionDefinition, Kind, Signature, StructDefinition,
    StructFieldInformation, TypeSignature,
};
use starcoin_vm_types::identifier::IdentStr;

pub trait Contract {
    fn check_compat(pre_version: &Self, new_version: &Self) -> Result<()>;

    fn compat_with(&self, pre_version: &Self) -> Result<()> {
        Self::check_compat(pre_version, &self)
    }

    fn is_compat_with(&self, pre_version: &Self) -> bool {
        self.compat_with(pre_version).is_ok()
    }
}

macro_rules! check_equal {
    ($check_name:literal, $pre:expr, $new:expr) => {
        if $pre != $new {
            return Err(anyhow!(
                "check compatibility by {} fail, pre: {:?}, new: {:?}",
                $check_name,
                $pre,
                $new
            ));
        }
    };
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModuleContract<'a> {
    address: &'a AccountAddress,
    name: &'a IdentStr,
    struct_contracts: Vec<StructContract<'a>>,
    function_contracts: Vec<FunctionContract<'a>>,
}

impl<'a> ModuleContract<'a> {
    pub fn new(compiled_module: &'a CompiledModule) -> Self {
        let module_handle = compiled_module.self_handle();
        Self {
            address: compiled_module.address_identifier_at(module_handle.address),
            name: compiled_module.identifier_at(module_handle.name),
            struct_contracts: compiled_module
                .struct_defs()
                .iter()
                .map(|struct_def| StructContract::new(compiled_module, struct_def))
                .collect(),
            function_contracts: compiled_module
                .function_defs()
                .iter()
                .filter(|fn_def| fn_def.is_public)
                .map(|fn_def| FunctionContract::new(compiled_module, fn_def))
                .collect(),
        }
    }

    pub fn find_struct(&self, name: &IdentStr) -> Option<&StructContract> {
        self.struct_contracts
            .iter()
            .find(|struct_ct| struct_ct.name == name)
    }

    pub fn find_fun(&self, name: &IdentStr) -> Option<&FunctionContract> {
        self.function_contracts
            .iter()
            .find(|fun_ct| fun_ct.name == name)
    }
}

impl<'a> Contract for ModuleContract<'a> {
    fn check_compat(pre_version: &Self, new_version: &Self) -> Result<()> {
        check_equal!("module_address", pre_version.address, new_version.address);
        check_equal!("module_name", pre_version.name, new_version.name);
        for pre_struct_ct in pre_version.struct_contracts.as_slice() {
            let new_struct_ct = new_version.find_struct(pre_struct_ct.name).ok_or_else(|| {
                format_err!(
                    "Can not find previous version struct {:?} in new version.",
                    pre_struct_ct
                )
            })?;
            new_struct_ct.compat_with(pre_struct_ct)?;
        }
        for pre_fun_ct in pre_version.function_contracts.as_slice() {
            let new_fun_ct = new_version.find_fun(pre_fun_ct.name).ok_or_else(|| {
                format_err!(
                    "Can not find previous version fun {:?} in new version.",
                    pre_fun_ct
                )
            })?;
            new_fun_ct.compat_with(pre_fun_ct)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StructContract<'a> {
    name: &'a IdentStr,
    is_nominal_resource: bool,
    type_parameters: &'a [Kind],
    fields: FieldsContract<'a>,
}

impl<'a> StructContract<'a> {
    pub fn new(compiled_module: &'a CompiledModule, struct_def: &'a StructDefinition) -> Self {
        let struct_handle = compiled_module.struct_handle_at(struct_def.struct_handle);
        Self {
            name: compiled_module.identifier_at(struct_handle.name),
            is_nominal_resource: struct_handle.is_nominal_resource,
            type_parameters: struct_handle.type_parameters.as_slice(),
            fields: FieldsContract::new(compiled_module, &struct_def.field_information),
        }
    }
}

impl<'a> Contract for StructContract<'a> {
    fn check_compat(pre_version: &Self, new_version: &Self) -> Result<()> {
        check_equal!("struct_name", pre_version.name, new_version.name);
        check_equal!(
            "struct_is_nominal_resource",
            pre_version.is_nominal_resource,
            new_version.is_nominal_resource
        );
        check_equal!(
            "struct_type_parameters",
            pre_version.type_parameters,
            new_version.type_parameters
        );
        new_version.fields.compat_with(&pre_version.fields)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FieldsContract<'a> {
    Native,
    Declared(Vec<FieldContract<'a>>),
}

impl<'a> FieldsContract<'a> {
    pub fn new(
        compiled_module: &'a CompiledModule,
        fields_info: &'a StructFieldInformation,
    ) -> Self {
        match fields_info {
            StructFieldInformation::Native => FieldsContract::Native,
            StructFieldInformation::Declared(fields) => FieldsContract::Declared(
                fields
                    .iter()
                    .map(|field| FieldContract::new(compiled_module, field))
                    .collect(),
            ),
        }
    }
}

impl<'a> Contract for FieldsContract<'a> {
    fn check_compat(pre_version: &Self, new_version: &Self) -> Result<()> {
        match (pre_version, new_version) {
            (FieldsContract::Native, FieldsContract::Native) => Ok(()),
            (FieldsContract::Declared(pre_fields), FieldsContract::Declared(new_fields)) => {
                if pre_fields.len() != new_fields.len() {
                    return Err(format_err!(
                        "{:?} not compat with {:?}, fields len not equals.",
                        pre_fields,
                        new_fields
                    ));
                }
                for (pre_field, new_field) in pre_fields.iter().zip(new_fields) {
                    new_field.compat_with(pre_field)?;
                }
                Ok(())
            }
            (pre, new) => Err(format_err!("{:?} not compat with {:?}", new, pre)),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FieldContract<'a> {
    signature: &'a TypeSignature,
}

impl<'a> FieldContract<'a> {
    pub fn new(_compiled_module: &'a CompiledModule, field_def: &'a FieldDefinition) -> Self {
        Self {
            signature: &field_def.signature,
        }
    }
}

impl<'a> Contract for FieldContract<'a> {
    fn check_compat(pre_version: &Self, new_version: &Self) -> Result<()> {
        check_equal!(
            "field_signature",
            pre_version.signature,
            new_version.signature
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FunctionContract<'a> {
    name: &'a IdentStr,
    parameters: &'a Signature,
    return_: &'a Signature,
    type_parameters: &'a [Kind],
}

impl<'a> FunctionContract<'a> {
    pub fn new(compiled_module: &'a CompiledModule, fun_def: &FunctionDefinition) -> Self {
        debug_assert!(fun_def.is_public);
        let fun_handle = compiled_module.function_handle_at(fun_def.function);
        Self {
            name: compiled_module.identifier_at(fun_handle.name),
            parameters: compiled_module.signature_at(fun_handle.parameters),
            return_: compiled_module.signature_at(fun_handle.return_),
            type_parameters: fun_handle.type_parameters.as_slice(),
        }
    }
}

impl<'a> Contract for FunctionContract<'a> {
    fn check_compat(pre_version: &Self, new_version: &Self) -> Result<()> {
        check_equal!("function", pre_version, new_version);
        Ok(())
    }
}
