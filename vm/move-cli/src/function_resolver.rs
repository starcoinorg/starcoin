use anyhow::{anyhow, Result};
use move_vm_runtime::data_cache::MoveStorage;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::file_format::{CompiledModule, FunctionDefinitionIndex};

pub trait FunctionResolver: MoveStorage {
    /// Return the name of the function at `idx` in `module_id`
    fn resolve_function(&self, module_id: &ModuleId, idx: u16) -> Result<Identifier> {
        let m = CompiledModule::deserialize(
            &self
                .get_module(module_id)
                .map_err(|e| e.into_vm_status())?
                .ok_or_else(|| anyhow!("Can't find module {:?}", module_id))?,
        )
        .map_err(|e| anyhow!("Failure deserializing module {:?}: {:?}", module_id, e))?;

        Ok(m.identifier_at(
            m.function_handle_at(m.function_def_at(FunctionDefinitionIndex(idx)).function)
                .name,
        )
        .to_owned())
    }
}

impl<R> FunctionResolver for R where R: MoveStorage {}
