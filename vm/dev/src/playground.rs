// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_abi_resolver::ABIResolver;
use starcoin_abi_types::TypeInstantiation;
use starcoin_crypto::HashValue;
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};
use starcoin_state_api::StateNodeStore;
use starcoin_statedb::ChainStateDB;
use starcoin_vm_runtime::metrics::VMMetrics;
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm_types::identifier::{IdentStr, Identifier};
use starcoin_vm_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::transaction::{DryRunTransaction, TransactionOutput};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::transaction_argument::TransactionArgument;
use starcoin_vm_types::vm_status::VMStatus;
use std::sync::Arc;

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

pub fn view_resource(
    state_view: &dyn StateView,
    struct_tag: StructTag,
    data: &[u8],
) -> Result<AnnotatedMoveStruct> {
    let annotator = MoveValueAnnotator::new(state_view);
    let value = annotator.view_struct(struct_tag, data)?;
    Ok(value)
}

pub fn dry_run(
    state_view: &dyn StateView,
    txn: DryRunTransaction,
    metrics: Option<VMMetrics>,
) -> Result<(VMStatus, TransactionOutput)> {
    let mut vm = StarcoinVM::new(metrics);
    vm.dry_run_transaction(state_view, txn)
}

pub fn call_contract(
    state_view: &dyn StateView,
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
