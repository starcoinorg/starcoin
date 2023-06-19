// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use starcoin_abi_decoder::decode_move_value;
use starcoin_abi_resolver::ABIResolver;
use starcoin_abi_types::TypeInstantiation;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::module_cache::ModuleCache;
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};
use starcoin_rpc_api::types::{DryRunOutputView, TransactionOutputView, WriteOpValueView};
use starcoin_state_api::StateNodeStore;
use starcoin_statedb::ChainStateDB;
use starcoin_vm_runtime::metrics::VMMetrics;
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::identifier::{IdentStr, Identifier};
use starcoin_vm_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{DryRunTransaction, TransactionOutput, TransactionPayload};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::transaction_argument::TransactionArgument;
use starcoin_vm_types::vm_status::VMStatus;
use std::sync::Arc;
use starcoin_vm_runtime::data_cache::{AsMoveResolver, StateViewCache};

#[derive(Clone)]
pub struct PlaygroudService {
    state: Arc<dyn StateNodeStore>,
    pub metrics: Option<VMMetrics>,
}

impl PlaygroudService {
    pub fn new(state_store: Arc<dyn StateNodeStore>, metrics: Option<VMMetrics>) -> Self {
        Self {
            state: state_store,
            metrics,
        }
    }
}

impl PlaygroudService {
    pub fn dry_run(
        &self,
        state_root: HashValue,
        txn: DryRunTransaction,
    ) -> Result<(VMStatus, TransactionOutput)> {
        let state_view = ChainStateDB::new(self.state.clone(), Some(state_root));
        dry_run(&state_view, txn, self.metrics.clone())
    }

    pub fn call_contract(
        &self,
        state_root: HashValue,
        module_id: ModuleId,
        func: Identifier,
        type_args: Vec<TypeTag>,
        args: Vec<TransactionArgument>,
    ) -> Result<Vec<AnnotatedMoveValue>> {
        let state_view = ChainStateDB::new(self.state.clone(), Some(state_root));
        let rets = call_contract(
            &state_view,
            module_id,
            func.as_str(),
            type_args,
            args,
            self.metrics.clone(),
        )?;
        let annotator = MoveValueAnnotator::new(&state_view);
        rets.into_iter()
            .map(|(ty, v)| annotator.view_value(&ty, &v))
            .collect::<Result<Vec<_>>>()
    }
    pub fn view_resource(
        &self,
        state_root: HashValue,
        struct_tag: &StructTag,
        data: &[u8],
    ) -> Result<AnnotatedMoveStruct> {
        let state_view = ChainStateDB::new(self.state.clone(), Some(state_root));
        view_resource(&state_view, struct_tag.clone(), data)
    }
}

pub fn view_resource<S: StateView>(
    state_view: &S,
    struct_tag: StructTag,
    data: &[u8],
) -> Result<AnnotatedMoveStruct> {
    let annotator = MoveValueAnnotator::new(state_view);
    let value = annotator.view_struct(struct_tag, data)?;
    Ok(value)
}

pub fn dry_run<S: StateView>(
    state_view: &S,
    txn: DryRunTransaction,
    metrics: Option<VMMetrics>,
) -> Result<(VMStatus, TransactionOutput)> {
    let mut vm = StarcoinVM::new(metrics);
    let state_view_cache = StateViewCache::new(state_view);
    vm.dry_run_transaction(&state_view_cache.as_move_resolver(), txn)
}

pub fn dry_run_explain<S: StateView>(
    state_view: &S,
    txn: DryRunTransaction,
    metrics: Option<VMMetrics>,
) -> anyhow::Result<DryRunOutputView> {
    let (vm_status, output) = dry_run(state_view, txn.clone(), metrics)?;
    let vm_status_explain = vm_status_translator::explain_vm_status(state_view, vm_status)?;
    let mut txn_output: TransactionOutputView = output.into();

    let resolver = {
        let module_cache = ModuleCache::new();
        // If the txn is package txn, we need to use modules in the package to resolve transaction output.
        if let TransactionPayload::Package(p) = txn.raw_txn.into_payload() {
            let modules = p
                .modules()
                .iter()
                .map(|m| CompiledModule::deserialize(m.code()))
                .collect::<Result<Vec<_>, _>>()?;
            for m in modules {
                module_cache.insert(m.self_id(), m);
            }
        }
        ABIResolver::new_with_module_cache(state_view, module_cache)
    };
    for action in txn_output.write_set.iter_mut() {
        let access_path = action.access_path.clone();
        if let Some(value) = &mut action.value {
            match value {
                WriteOpValueView::Code(view) => {
                    view.abi = Some(resolver.resolve_module_code(view.code.0.as_slice())?);
                }
                WriteOpValueView::Resource(view) => {
                    let struct_tag = access_path.path.as_struct_tag().ok_or_else(|| {
                        format_err!("invalid resource access path: {}", access_path)
                    })?;
                    let struct_abi = resolver.resolve_struct_tag(struct_tag)?;
                    view.json = Some(decode_move_value(
                        &TypeInstantiation::Struct(Box::new(struct_abi)),
                        view.raw.0.as_slice(),
                    )?)
                }
            }
        }
    }
    Ok(DryRunOutputView {
        explained_status: vm_status_explain,
        txn_output,
    })
}

pub fn call_contract<S: StateView>(
    state_view: &S,
    module_id: ModuleId,
    func: &str,
    type_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
    metrics: Option<VMMetrics>,
) -> Result<Vec<(TypeTag, Vec<u8>)>> {
    let function_name = IdentStr::new(func)?;
    let abi_resolver = ABIResolver::new(state_view);
    let func_abi = abi_resolver.resolve_function(&module_id, function_name)?;

    // validate params
    {
        anyhow::ensure!(
            func_abi.ty_args().len() == type_args.len(),
            "type args length mismatch, expect {}, actual {}",
            func_abi.ty_args().len(),
            type_args.len()
        );
        let arg_abi = func_abi.args();
        anyhow::ensure!(
            arg_abi.len() == args.len(),
            "args length mismatch, expect {}, actual {}",
            arg_abi.len(),
            args.len()
        );
    }

    let ty_tags_abi = type_args
        .as_slice()
        .iter()
        .map(|t| abi_resolver.resolve_type_tag(t))
        .collect::<Result<Vec<_>>>()?;
    let func_instantiation = func_abi.instantiation(&ty_tags_abi)?;

    // after instantiate the function, we check the arg types.
    {
        for (i, (abi, v)) in func_instantiation.args().iter().zip(&args).enumerate() {
            match (abi.type_abi(), &v) {
                (TypeInstantiation::U8, TransactionArgument::U8(_))
                | (TypeInstantiation::U64, TransactionArgument::U64(_))
                | (TypeInstantiation::U128, TransactionArgument::U128(_))
                | (TypeInstantiation::U16, TransactionArgument::U16(_))
                | (TypeInstantiation::U32, TransactionArgument::U32(_))
                | (TypeInstantiation::U256, TransactionArgument::U256(_))
                | (TypeInstantiation::Address, TransactionArgument::Address(_))
                | (TypeInstantiation::Bool, TransactionArgument::Bool(_)) => {}
                (TypeInstantiation::Vector(sub_ty), TransactionArgument::U8Vector(_))
                    if sub_ty.as_ref() == &TypeInstantiation::U8 => {}
                (TypeInstantiation::Reference(_, ref_type), TransactionArgument::Address(_))
                    if (ref_type.as_ref() == &TypeInstantiation::Address)
                        || (ref_type.as_ref() == &TypeInstantiation::Signer) => {}
                (abi, value) => anyhow::bail!(
                    "arg type at position {} mismatch, expect {:?}, actual {}",
                    i,
                    abi,
                    value
                ),
            }
        }
    }

    let mut vm = StarcoinVM::new(metrics);
    let rets = vm.execute_readonly_function(
        state_view,
        &module_id,
        function_name,
        type_args,
        convert_txn_args(&args),
    )?;

    let ret_tys = func_instantiation
        .returns()
        .iter()
        .map(|r| r.type_tag())
        .collect::<Result<Vec<_>>>()?;
    anyhow::ensure!(
        ret_tys.len() == rets.len(),
        "length of return values mismatch, expect: {}, got: {}",
        ret_tys.len(),
        rets.len()
    );
    Ok(ret_tys.into_iter().zip(rets).collect())
}
