use crate::resolver::Resolver;
use anyhow::Result;
use starcoin_vm_types::abi::{FieldABI, ModuleABI, ScriptFunctionABI, StructABI, TypeABI};
use starcoin_vm_types::language_storage::{FunctionId, ModuleId, StructTag, TypeTag};

pub struct ABIResolver<'a> {
    resolver: Resolver<'a>,
}

impl<'a> ABIResolver<'a> {
    pub fn resolve_module(&self, module_id: &ModuleId) -> Result<ModuleABI> {
        let module = self
            .resolver
            .get_module(module_id.address(), module_id.name())?;

        todo!()
    }
    pub fn resolve_struct_tag(&self, struct_tag: &StructTag) -> Result<StructABI> {
        let fat_struct_type = self.resolver.resolve_struct(struct_tag)?;
        let field_names = self.resolver.get_field_names(&fat_struct_type)?;
        let mut fields = Vec::with_capacity(field_names.len());
        for (layout, field) in fat_struct_type.layout.iter().zip(field_names.iter()) {
            let field = FieldABI::new(
                field.to_string(),
                String::new(),
                self.resolve_type_tag(&layout.type_tag().map_err(|e| anyhow::anyhow!("{:?}", e))?)?,
            );
            fields.push(field);
        }
        Ok(StructABI::new(
            fat_struct_type.name.to_string(),
            ModuleId::new(fat_struct_type.address, fat_struct_type.module),
            String::new(),
            fields,
        ))
    }
    pub fn resolve_type_tag(&self, type_tag: &TypeTag) -> Result<TypeABI> {
        Ok(match type_tag {
            TypeTag::Bool => TypeABI::Bool,
            TypeTag::U8 => TypeABI::U8,
            TypeTag::U64 => TypeABI::U64,
            TypeTag::U128 => TypeABI::U128,
            TypeTag::Address => TypeABI::Address,

            TypeTag::Signer => TypeABI::Signer,
            TypeTag::Vector(sub_type) => TypeABI::new_vector(self.resolve_type_tag(&sub_type)?),
            TypeTag::Struct(struct_type) => {
                TypeABI::new_struct(self.resolve_struct_tag(&struct_type)?)
            }
        })
    }

    pub fn resolve_function(&self, function_id: FunctionId) -> Result<ScriptFunctionABI> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
