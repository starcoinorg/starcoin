use anyhow::Result;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::file_format::{CompiledModule, FunctionDefinitionIndex};
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::vm_status::{AbortLocation, StatusCode, VMStatus};
use std::convert::TryFrom;
use std::fmt;

pub fn locate_execution_failure(
    state: &dyn StateView,
    location: AbortLocation,
    function: u16,
) -> Result<Option<(ModuleId, Identifier)>> {
    let module = match location {
        AbortLocation::Module(module_id) => {
            let ap =
                AccessPath::code_access_path(*module_id.address(), module_id.name().to_owned());

            match state.get(&ap)? {
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

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, JsonSchema)]
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

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, JsonSchema)]
pub enum VmStatusExplainView {
    /// The VM status corresponding to an EXECUTED status code
    Executed,
    /// Indicates an error from the VM, e.g. OUT_OF_GAS, INVALID_AUTH_KEY, RET_TYPE_MISMATCH_ERROR
    /// etc.
    /// The code will neither EXECUTED nor ABORTED
    Error(String),

    /// Indicates an `abort` from inside Move code. Contains the location of the abort and the code
    MoveAbort {
        //TODO:remote define it
        #[schemars(with = "String")]
        location: AbortLocation,
        abort_code: u64,
        explain: MoveAbortExplain,
    },

    /// Indicates an failure from inside Move code, where the VM could not continue exection, e.g.
    /// dividing by zero or a missing resource
    ExecutionFailure {
        /// status_code in str.
        status_code: String,
        /// status_code in u64.
        status: u64,
        #[schemars(with = "String")]
        location: AbortLocation,
        function: u16,
        function_name: Option<String>,
        code_offset: u16,
    },
}

/// custom debug for VmStatusExplainView for usage in functional test
impl fmt::Debug for VmStatusExplainView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Executed => fmt::Debug::fmt(&VMStatus::Executed, f),
            Self::Error(code) => f.debug_struct("ERROR").field("status_code", code).finish(),
            Self::MoveAbort {
                location,
                abort_code,
                ..
            } => fmt::Debug::fmt(&VMStatus::MoveAbort(location.clone(), *abort_code), f),
            Self::ExecutionFailure {
                status,
                location,
                function,
                code_offset,
                ..
            } => fmt::Debug::fmt(
                &VMStatus::ExecutionFailure {
                    status_code: StatusCode::try_from(*status).unwrap(),
                    location: location.clone(),
                    function: *function,
                    code_offset: *code_offset,
                },
                f,
            ),
        }
    }
}

pub fn explain_vm_status(
    state_view: &dyn StateView,
    vm_status: VMStatus,
) -> Result<VmStatusExplainView> {
    let vm_status_explain = match &vm_status {
        VMStatus::Executed => VmStatusExplainView::Executed,
        VMStatus::Error(c) => VmStatusExplainView::Error(format!("{:?}", c)),
        VMStatus::MoveAbort(location, abort_code) => VmStatusExplainView::MoveAbort {
            location: location.clone(),
            abort_code: *abort_code,
            explain: explain_move_abort(location.clone(), *abort_code),
        },
        VMStatus::ExecutionFailure {
            status_code,
            location,
            function,
            code_offset,
        } => VmStatusExplainView::ExecutionFailure {
            status_code: format!("{:?}", status_code),
            status: (*status_code).into(),
            location: location.clone(),
            function: *function,
            function_name: locate_execution_failure(state_view, location.clone(), *function)?
                .map(|l| l.1.to_string()),
            code_offset: *code_offset,
        },
    };
    Ok(vm_status_explain)
}

// //should define a TransactionStatusExplainView?
// pub fn explain_transaction_status(
//     state_view: &dyn StateView,
//     txn_status: TransactionStatus,
// ) -> Result<VmStatusExplainView> {
//     let vm_status_explain = match &txn_status {
//         TransactionStatus::Keep(keep_status) => match keep_status {
//             KeptVMStatus::Executed => VmStatusExplainView::Executed,
//             KeptVMStatus::MoveAbort(location, abort_code) => VmStatusExplainView::MoveAbort {
//                 location: location.clone(),
//                 abort_code: *abort_code,
//                 explain: explain_move_abort(location.clone(), *abort_code),
//             },
//             KeptVMStatus::ExecutionFailure {
//                 location,
//                 function,
//                 code_offset,
//             } => VmStatusExplainView::ExecutionFailure {
//                 status_code: "".to_string(),
//                 location: location.clone(),
//                 function: *function,
//                 function_name: locate_execution_failure(state_view, location.clone(), *function)?
//                     .map(|l| l.1.to_string()),
//                 code_offset: *code_offset,
//             },
//             c => VmStatusExplainView::Error(format!("{:?}", c)),
//         },
//         TransactionStatus::Discard(c) => VmStatusExplainView::Error(format!("{:?}", c)),
//     };
//     Ok(vm_status_explain)
// }
