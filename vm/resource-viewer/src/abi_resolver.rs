use crate::resolver::Resolver;
use anyhow::anyhow;
use anyhow::Result;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{FunctionDefinitionIndex, StructDefinitionIndex, Visibility};
use move_binary_format::normalized::{Struct, Type};
use move_binary_format::CompiledModule;
use starcoin_vm_types::abi::ArgumentABI;
use starcoin_vm_types::abi::{
    FieldABI, ModuleABI, ScriptFunctionABI, StructABI, TypeABI, TypeArgumentABI,
};
use starcoin_vm_types::identifier::{IdentStr, Identifier};
use starcoin_vm_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_vm_types::state_view::StateView;

pub struct ABIResolver<'a> {
    resolver: Resolver<'a>,
}

impl<'a> ABIResolver<'a> {
    pub fn new(state: &'a dyn StateView) -> Self {
        Self {
            resolver: Resolver::new(state),
        }
    }

    pub fn resolve_module(&self, module_id: &ModuleId) -> Result<ModuleABI> {
        let module = self
            .resolver
            .get_module(module_id.address(), module_id.name())?;
        let m = move_binary_format::normalized::Module::new(&module);
        let structs = m
            .structs
            .iter()
            .map(|(name, s)| self.struct_to_abi(module_id, name, s))
            .collect::<Result<Vec<_>>>()?;
        let functions = m
            .exposed_functions
            .iter()
            .filter(|(_, func)| func.visibility == Visibility::Script) // only script functions
            .map(|(name, func)| self.function_to_abi(module_id, name.as_ident_str(), func))
            .collect::<Result<Vec<_>>>()?;
        Ok(ModuleABI::new(m.module_id(), structs, functions))
    }

    pub fn resolve_struct_tag(&self, struct_tag: &StructTag) -> Result<StructABI> {
        let struct_abi =
            self.resolve_struct(&struct_tag.module_id(), struct_tag.name.as_ident_str())?;
        let ty_args = struct_tag
            .type_params
            .iter()
            .map(|ty| self.resolve_type_tag(ty))
            .collect::<Result<Vec<_>>>()?;
        struct_abi.subst(&ty_args)
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

    pub fn resolve_struct(&self, module_id: &ModuleId, name: &IdentStr) -> Result<StructABI> {
        let module = self
            .resolver
            .get_module(module_id.address(), module_id.name())?;
        let struct_def = find_struct_def_in_module(module.as_ref(), name)?;
        let (name, s) =
            move_binary_format::normalized::Struct::new(&module, module.struct_def_at(struct_def));
        self.struct_to_abi(module_id, &name, &s)
    }
    pub fn resolve_type(&self, ty: &Type) -> Result<TypeABI> {
        Ok(match ty {
            Type::Bool => TypeABI::Bool,
            Type::U8 => TypeABI::U8,
            Type::U64 => TypeABI::U64,
            Type::U128 => TypeABI::U128,
            Type::Address => TypeABI::Address,
            Type::Signer => TypeABI::Signer,
            Type::Struct {
                address,
                module,
                name,
                type_arguments,
            } => {
                let struct_abi = self.resolve_struct(
                    &ModuleId::new(*address, module.clone()),
                    name.as_ident_str(),
                )?;
                let type_args = type_arguments
                    .iter()
                    .map(|t| self.resolve_type(t))
                    .collect::<Result<Vec<_>>>()?;
                let inst_struct_abi = struct_abi.subst(&type_args)?;
                TypeABI::Struct(inst_struct_abi)
            }
            Type::Vector(sub_ty) => TypeABI::new_vector(self.resolve_type(&sub_ty)?),
            Type::TypeParameter(i) => TypeABI::TypeParameter(*i as usize),
            Type::Reference(_) | Type::MutableReference(_) => {
                anyhow::bail!("")
            }
        })
    }
    pub fn resolve_function(
        &self,
        module_id: &ModuleId,
        function_name: &IdentStr,
    ) -> Result<ScriptFunctionABI> {
        let module = self
            .resolver
            .get_module(module_id.address(), module_id.name())?;
        let function_def_idx = find_function_def_in_module(module.as_ref(), function_name)?;
        let function_def = module.function_def_at(function_def_idx);
        let (_func_name, func) =
            move_binary_format::normalized::Function::new(module.as_ref(), function_def);
        self.function_to_abi(module_id, function_name, &func)
    }

    /// resolve function with concrete type args.
    pub fn resolve_function_instantiation(
        &self,
        module_id: &ModuleId,
        function_name: &IdentStr,
        type_args: &[TypeTag],
    ) -> Result<ScriptFunctionABI> {
        let script_function_abi = self.resolve_function(module_id, function_name)?;
        let type_args = type_args
            .iter()
            .map(|t| self.resolve_type_tag(t))
            .collect::<Result<Vec<_>>>()?;
        Ok(ScriptFunctionABI::new(
            script_function_abi.name().to_string(),
            script_function_abi.module_name().clone(),
            script_function_abi.doc().to_string(),
            script_function_abi.ty_args().to_vec(),
            script_function_abi
                .args()
                .iter()
                .map(|arg| {
                    Ok(ArgumentABI::new(
                        arg.name().to_string(),
                        arg.type_abi().subst(&type_args)?,
                        arg.doc().to_string(),
                    ))
                })
                .collect::<Result<Vec<_>>>()?,
        ))
    }

    fn struct_to_abi(
        &self,
        module_id: &ModuleId,
        name: &Identifier,
        s: &Struct,
    ) -> Result<StructABI> {
        let fields: Result<Vec<_>> = s
            .fields
            .iter()
            .map(|f| {
                Ok(FieldABI::new(
                    f.name.to_string(),
                    String::new(),
                    self.resolve_type(&f.type_)?,
                ))
            })
            .collect();
        let type_parameters = s
            .type_parameters
            .iter()
            .enumerate()
            .map(|(i, ab)| TypeArgumentABI::new(format!("T{}", i), *ab))
            .collect();
        let abi = StructABI::new(
            name.to_string(),
            module_id.clone(),
            String::new(),
            type_parameters,
            fields?,
            s.abilities,
        );
        Ok(abi)
    }

    fn function_to_abi(
        &self,
        module_id: &ModuleId,
        name: &IdentStr,
        func: &move_binary_format::normalized::Function,
    ) -> Result<ScriptFunctionABI> {
        let type_parameters = func
            .type_parameters
            .iter()
            .enumerate()
            .map(|(i, ab)| TypeArgumentABI::new(format!("T{}", i), *ab))
            .collect();
        let parameters = func
            .parameters
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let ty = self.resolve_type(t)?;
                Ok(ArgumentABI::new(format!("p{}", i), ty, String::new()))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(ScriptFunctionABI::new(
            name.to_string(),
            module_id.clone(),
            String::new(),
            type_parameters,
            parameters,
        ))
    }
}

fn find_function_def_in_module(
    module: &CompiledModule,
    name: &IdentStr,
) -> Result<FunctionDefinitionIndex> {
    for (i, defs) in module.function_defs().iter().enumerate() {
        let func_handle = module.function_handle_at(defs.function);
        if module.identifier_at(func_handle.name) == name {
            return Ok(FunctionDefinitionIndex::new(i as u16));
        }
    }
    Err(anyhow!(
        "Function {:?} not found in {:?}",
        name,
        module.self_id()
    ))
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

#[cfg(test)]
mod tests {
    use crate::abi_resolver::ABIResolver;
    use anyhow::Result;
    use move_binary_format::CompiledModule;
    use starcoin_vm_types::access_path::{AccessPath, DataPath};
    use starcoin_vm_types::account_config::genesis_address;
    use starcoin_vm_types::identifier::Identifier;
    use starcoin_vm_types::language_storage::ModuleId;
    use starcoin_vm_types::parser::parse_struct_tag;
    use starcoin_vm_types::state_view::StateView;
    use std::collections::BTreeMap;

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
    impl StateView for InMemoryStateView {
        fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
            let module_id = match &access_path.path {
                DataPath::Code(name) => ModuleId::new(access_path.address, name.clone()),
                _ => anyhow::bail!("no data"),
            };
            Ok(self.modules.get(&module_id).map(|m| {
                let mut data = vec![];
                m.serialize(&mut data).unwrap();
                data
            }))
        }

        fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
            todo!()
        }

        fn is_genesis(&self) -> bool {
            todo!()
        }
    }

    #[test]
    fn test_resolver_abi() {
        let modules = stdlib::load_latest_stable_compiled_modules().unwrap().1;
        let view = InMemoryStateView::new(modules);
        let r = ABIResolver::new(&view);
        // test module ok
        {
            let m = ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap());
            r.resolve_module(&m).unwrap();
        }
        // test struct tag
        {
            let st = parse_struct_tag(
                "0x1::Dao::Proposal<0x1::STC::STC, 0x1::MintDaoProposal::MintToken>",
            )
            .unwrap();
            r.resolve_struct_tag(&st).unwrap();
        }
        // test struct def
        {
            let m = ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap());
            let s = Identifier::new("Proposal").unwrap();
            let func_abi = r.resolve_struct(&m, s.as_ident_str()).unwrap();
            println!("{}", serde_json::to_string_pretty(&func_abi).unwrap());
        }
        // test function
        {
            let m = ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap());
            let func = Identifier::new("queue_proposal_action").unwrap();
            let func_abi = r.resolve_function(&m, func.as_ident_str()).unwrap();
            println!("{}", serde_json::to_string_pretty(&func_abi).unwrap());
        }
    }

    #[test]
    fn test_normalized() {
        let modules = stdlib::load_latest_stable_compiled_modules().unwrap().1;
        let dao = modules
            .iter()
            .find(|m| {
                m.self_id() == ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap())
            })
            .unwrap();
        let _m = move_binary_format::normalized::Module::new(dao);
    }
}
