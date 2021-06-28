use anyhow::Result;
use serde::Serialize;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::file_format::{CompiledModule, FunctionDefinitionIndex};
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::vm_status::AbortLocation;

pub struct VmStatusTranslator<M: StateView> {
    state: M,
}

impl<M: StateView> VmStatusTranslator<M> {
    pub fn new(s: M) -> Self {
        Self { state: s }
    }

    pub fn locate_execution_failure(
        &self,
        location: AbortLocation,
        function: u16,
    ) -> Result<Option<(ModuleId, Identifier)>> {
        let module = match location {
            AbortLocation::Module(module_id) => {
                let ap =
                    AccessPath::code_access_path(*module_id.address(), module_id.name().to_owned());

                match self.state.get(&ap)? {
                    Some(bytes) => CompiledModule::deserialize(&bytes).ok(),
                    None => None,
                }
            }
            AbortLocation::Script => None,
        };
        Ok(match module {
            Some(m) => {
                let fd = m.function_def_at(FunctionDefinitionIndex(function));
                let fh = m.function_handle_at(fd.function);
                let func_name = m.identifier_at(fh.name).to_owned();

                Some((m.self_id(), func_name))
            }
            None => None,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MoveAbortExplain {
    pub category_code: u64,
    pub category_name: Option<String>,
    pub reason_code: u64,
    pub reason_name: Option<String>,
}

pub fn explain_move_abort(abort_location: AbortLocation, abort_code: u64) -> MoveAbortExplain {
    let category = abort_code & 0xFFu64;
    let reason_code = abort_code >> 8;

    let err_context = match abort_location {
        AbortLocation::Module(module_id) => {
            starcoin_move_explain::get_explanation(&module_id, abort_code)
        }
        AbortLocation::Script => None,
    };
    match err_context {
        Some(ctx) => MoveAbortExplain {
            category_code: category,
            category_name: Some(ctx.category.code_name),
            reason_code,
            reason_name: Some(ctx.reason.code_name),
        },
        None => MoveAbortExplain {
            category_code: category,
            category_name: None,
            reason_code,
            reason_name: None,
        },
    }
}
