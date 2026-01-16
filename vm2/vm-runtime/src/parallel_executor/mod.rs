// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod storage_wrapper;
mod vm_wrapper;

use crate::{
    data_cache::StateViewCache,
    parallel_executor::{storage_wrapper::DelayedFieldCache, vm_wrapper::StarcoinVMWrapper},
    preprocess_transaction,
    starcoin_vm::StarcoinVM,
    PreprocessedTransaction,
};
use bytes::Bytes;
use move_binary_format::errors::PartialVMError;
use move_core_types::language_storage::StructTag;
use move_core_types::value::MoveTypeLayout;
use move_core_types::vm_status::{StatusCode, VMStatus};
use move_vm_types::delayed_values::delayed_field_id::{DelayedFieldID, ExtractWidth};
use move_vm_types::value_serde::{
    deserialize_and_allow_delayed_values, serialize_and_replace_ids_with_values,
    ValueToIdentifierMapping,
};
use move_vm_types::value_traversal::find_identifiers_in_value;
use rayon::prelude::*;
use starcoin_aggregator::types::ReadPosition;
use starcoin_logger::prelude::info;
use starcoin_metrics::metrics::VMMetrics;
use starcoin_mvhashmap::versioned_delayed_fields::{
    TVersionedDelayedFieldView, VersionedDelayedFields,
};
use starcoin_parallel_executor::{
    errors::Error,
    executor::ParallelTransactionExecutor,
    task::{Transaction as PTransaction, TransactionOutput as PTransactionOutput},
};
use starcoin_vm_runtime_types::abstract_write_op::{
    AbstractResourceWriteOp, GroupWrite, InPlaceDelayedFieldChangeOp,
    ResourceGroupInPlaceDelayedFieldChangeOp, WriteWithDelayedFieldsOp,
};
use starcoin_vm_runtime_types::output::VMOutput;
use starcoin_vm_runtime_types::resolver::ResourceGroupSize;
use starcoin_vm_types::{
    contract_event::ContractEvent,
    on_chain_config::{Features, OnChainConfig},
    state_store::state_key::StateKey,
    state_store::StateView,
    transaction::{Transaction, TransactionOutput, TransactionStatus},
    write_set::WriteOp,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ParallelStateKey {
    Resource(StateKey),
    GroupMember { group_key: StateKey, tag: StructTag },
    GroupSize(StateKey),
}

#[derive(Clone, Debug)]
pub enum ParallelStateValue {
    Write(WriteOp),
    GroupSize(ResourceGroupSize),
}

impl ParallelStateValue {
    pub fn as_write_op(&self) -> Option<&WriteOp> {
        match self {
            Self::Write(write_op) => Some(write_op),
            Self::GroupSize(_) => None,
        }
    }

    pub fn into_write_op(self) -> Option<WriteOp> {
        match self {
            Self::Write(write_op) => Some(write_op),
            Self::GroupSize(_) => None,
        }
    }

    pub fn as_group_size(&self) -> Option<ResourceGroupSize> {
        match self {
            Self::GroupSize(size) => Some(*size),
            Self::Write(_) => None,
        }
    }
}

impl PTransaction for PreprocessedTransaction {
    type Key = ParallelStateKey;
    type Value = ParallelStateValue;

    fn is_block_prologue(&self) -> bool {
        matches!(self, PreprocessedTransaction::BlockMetadata(_))
    }

    fn is_block_epilogue(&self) -> bool {
        matches!(self, PreprocessedTransaction::BlockEpilogue(..))
    }
}

// Wrapper to avoid orphan rule
pub(crate) struct StarcoinTransactionOutput {
    output: VMOutput,
    group_read_layouts: HashMap<StateKey, BTreeMap<StructTag, Arc<MoveTypeLayout>>>,
}

impl StarcoinTransactionOutput {
    pub fn new(
        output: VMOutput,
        group_read_layouts: HashMap<StateKey, BTreeMap<StructTag, Arc<MoveTypeLayout>>>,
    ) -> Self {
        Self {
            output,
            group_read_layouts,
        }
    }

    pub fn into_inner(
        self,
    ) -> (
        VMOutput,
        HashMap<StateKey, BTreeMap<StructTag, Arc<MoveTypeLayout>>>,
    ) {
        (self.output, self.group_read_layouts)
    }

    #[allow(dead_code)]
    fn needs_sequential_materialization(&self) -> bool {
        if !self.output.aggregator_v1_delta_set().is_empty() {
            return true;
        }
        self.output.resource_write_set().values().any(|op| {
            matches!(
                op,
                AbstractResourceWriteOp::WriteResourceGroup(_)
                    | AbstractResourceWriteOp::ResourceGroupInPlaceDelayedFieldChange(_)
            )
        })
    }

    fn contains_delayed_fields(&self) -> bool {
        self.output.contains_delayed_fields()
    }
}

impl PTransactionOutput for StarcoinTransactionOutput {
    type T = PreprocessedTransaction;

    fn get_writes(&self) -> Vec<(ParallelStateKey, ParallelStateValue)> {
        let mut writes = Vec::new();
        for (key, op) in self.output.resource_write_set() {
            match op {
                AbstractResourceWriteOp::Write(write_op) => {
                    writes.push((
                        ParallelStateKey::Resource(key.clone()),
                        ParallelStateValue::Write(write_op.clone()),
                    ));
                }
                AbstractResourceWriteOp::WriteWithDelayedFields(WriteWithDelayedFieldsOp {
                    write_op,
                    ..
                }) => {
                    writes.push((
                        ParallelStateKey::Resource(key.clone()),
                        ParallelStateValue::Write(write_op.clone()),
                    ));
                }
                AbstractResourceWriteOp::WriteResourceGroup(group_write) => {
                    // Track group metadata updates alongside member writes.
                    writes.push((
                        ParallelStateKey::Resource(key.clone()),
                        ParallelStateValue::Write(group_write.metadata_op().clone()),
                    ));
                    for (tag, (inner_op, _layout)) in group_write.inner_ops() {
                        writes.push((
                            ParallelStateKey::GroupMember {
                                group_key: key.clone(),
                                tag: tag.clone(),
                            },
                            ParallelStateValue::Write(inner_op.clone()),
                        ));
                    }
                    let size = group_write
                        .maybe_group_op_size()
                        .unwrap_or(ResourceGroupSize::zero_combined());
                    writes.push((
                        ParallelStateKey::GroupSize(key.clone()),
                        ParallelStateValue::GroupSize(size),
                    ));
                }
                AbstractResourceWriteOp::InPlaceDelayedFieldChange(_)
                | AbstractResourceWriteOp::ResourceGroupInPlaceDelayedFieldChange(_) => {}
            }
        }
        for (key, write_op) in self.output.module_write_set() {
            writes.push((
                ParallelStateKey::Resource(key.clone()),
                ParallelStateValue::Write(write_op.clone()),
            ));
        }
        for (key, write_op) in self.output.aggregator_v1_write_set() {
            writes.push((
                ParallelStateKey::Resource(key.clone()),
                ParallelStateValue::Write(write_op.clone()),
            ));
        }
        writes
    }

    fn gas_used(&self) -> u64 {
        self.output.gas_used()
    }

    /// Execution output for transactions that comes after SkipRest signal.
    fn skip_output() -> Self {
        Self::new(
            VMOutput::empty_with_status(TransactionStatus::Retry),
            HashMap::new(),
        )
    }

    fn delayed_field_change_set(
        &self,
    ) -> Vec<(
        DelayedFieldID,
        starcoin_aggregator::delayed_change::DelayedChange<DelayedFieldID>,
    )> {
        self.output
            .delayed_field_change_set()
            .iter()
            .map(|(id, change)| (*id, change.clone()))
            .collect()
    }
}

pub struct ParallelStarcoinVM();

impl ParallelStarcoinVM {
    pub fn execute_block<S: StateView + Sync>(
        transactions: Vec<Transaction>,
        state_view: &S,
        concurrency_level: usize,
        block_gas_limit: Option<u64>,
        metrics: Option<VMMetrics>,
    ) -> Result<(Vec<TransactionOutput>, Option<Error<VMStatus>>), VMStatus> {
        let delayed_fields_enabled = Features::fetch_config(state_view)
            .unwrap_or_default()
            .is_aggregator_v2_delayed_fields_enabled();
        let signature_verified_block: Vec<PreprocessedTransaction> = transactions
            .par_iter()
            .map(|txn| preprocess_transaction(txn.clone()))
            .collect();

        let delayed_field_cache = Arc::new(DelayedFieldCache::default());
        let exec_start = std::time::Instant::now();
        match ParallelTransactionExecutor::<PreprocessedTransaction, StarcoinVMWrapper<S>>::new(
            concurrency_level,
            block_gas_limit,
        )
        .with_delayed_fields(delayed_fields_enabled)
        .execute_transactions_parallel_with_delayed_fields(
            (state_view, delayed_field_cache.clone()),
            signature_verified_block,
        ) {
            Ok((results, delayed_fields)) => {
                let exec_ms = exec_start.elapsed().as_secs_f64() * 1000.0;
                if !delayed_fields_enabled
                    && results
                        .iter()
                        .any(|(_, output)| output.contains_delayed_fields())
                {
                    return Err(VMStatus::error(
                        StatusCode::FEATURE_UNDER_GATING,
                        Some("delayed fields feature disabled".to_string()),
                    ));
                }
                let materialize_start = std::time::Instant::now();
                let outputs = materialize_parallel_outputs(
                    results,
                    delayed_fields,
                    delayed_field_cache,
                    state_view,
                )?;
                let materialize_ms = materialize_start.elapsed().as_secs_f64() * 1000.0;
                info!(
                    target: "vm-bench",
                    "parallel execute done: exec_ms={:.3} materialize_ms={:.3}",
                    exec_ms,
                    materialize_ms
                );
                Ok((outputs, None))
            }
            Err(err @ Error::BlockRestart) => {
                let output = StarcoinVM::execute_block_and_keep_vm_status(
                    transactions,
                    state_view,
                    block_gas_limit,
                    metrics,
                )?;
                Ok((
                    output
                        .into_iter()
                        .map(|(_vm_status, txn_output)| txn_output)
                        .collect(),
                    Some(err),
                ))
            }
            Err(Error::InvariantViolation) => Err(VMStatus::error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
                None,
            )),
            Err(Error::UserError(err)) => Err(err),
        }
    }
}

struct DelayedFieldValueMapping<'a> {
    delayed_fields: &'a VersionedDelayedFields<DelayedFieldID>,
    txn_idx: usize,
}

impl ValueToIdentifierMapping for DelayedFieldValueMapping<'_> {
    type Identifier = DelayedFieldID;

    fn value_to_identifier(
        &self,
        _kind: &move_core_types::value::IdentifierMappingKind,
        _layout: &MoveTypeLayout,
        _value: move_vm_types::values::Value,
    ) -> Result<Self::Identifier, PartialVMError> {
        Err(PartialVMError::new(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
        ))
    }

    fn identifier_to_value(
        &self,
        layout: &MoveTypeLayout,
        identifier: Self::Identifier,
    ) -> Result<move_vm_types::values::Value, PartialVMError> {
        let value = self
            .delayed_fields
            .read_latest_predicted_value(&identifier, self.txn_idx, ReadPosition::AfterCurrentTxn)
            .map_err(|err| {
                PartialVMError::new(StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR)
                    .with_message(format!(
                        "Failed to read delayed field {:?} at txn {}: {:?}",
                        identifier, self.txn_idx, err
                    ))
            })?;
        value.try_into_move_value(layout, identifier.extract_width())
    }
}

fn materialize_parallel_outputs<S: StateView + Sync>(
    outputs: Vec<(usize, StarcoinTransactionOutput)>,
    delayed_fields: VersionedDelayedFields<DelayedFieldID>,
    delayed_field_cache: Arc<DelayedFieldCache>,
    state_view: &S,
) -> Result<Vec<TransactionOutput>, VMStatus> {
    let mut outputs = outputs;
    let mut needs_sequential = false;
    let mut group_keys = HashSet::new();
    for (_, output) in outputs.iter() {
        if !output.output.aggregator_v1_delta_set().is_empty() {
            needs_sequential = true;
            break;
        }
        for (key, op) in output.output.resource_write_set() {
            if let AbstractResourceWriteOp::WriteResourceGroup(_) = op {
                if !group_keys.insert(key.clone()) {
                    needs_sequential = true;
                    break;
                }
            }
        }
        if needs_sequential {
            break;
        }
    }
    outputs.sort_by_key(|(idx, _)| *idx);

    if !needs_sequential {
        let mut results = outputs
            .into_par_iter()
            .map(|(txn_idx, output)| {
                let (vm_output, group_read_layouts) = output.into_inner();
                let has_delayed = vm_output.contains_delayed_fields();
                let has_group_ops = vm_output.resource_write_set().values().any(|op| {
                    matches!(
                        op,
                        AbstractResourceWriteOp::WriteResourceGroup(_)
                            | AbstractResourceWriteOp::ResourceGroupInPlaceDelayedFieldChange(_)
                    )
                });
                if !has_delayed && vm_output.aggregator_v1_delta_set().is_empty() && !has_group_ops
                {
                    let txn_output = vm_output.into_transaction_output().map_err(|err| {
                        VMStatus::error(
                            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                            Some(err.to_string()),
                        )
                    })?;
                    return Ok((txn_idx, txn_output));
                }
                let mapping = DelayedFieldValueMapping {
                    delayed_fields: &delayed_fields,
                    txn_idx,
                };

                let mut group_cache = HashMap::new();
                let patched_resource_write_set = if has_group_ops {
                    materialize_resource_write_set(
                        &vm_output,
                        &mapping,
                        &delayed_field_cache,
                        &group_read_layouts,
                        state_view,
                        &mut group_cache,
                        has_delayed,
                    )?
                } else {
                    materialize_resource_write_set_no_groups(
                        &vm_output,
                        &mapping,
                        &delayed_field_cache,
                    )?
                };
                let patched_events = if has_delayed {
                    materialize_events(&vm_output, &mapping)?
                } else {
                    vm_output
                        .events()
                        .iter()
                        .map(|(event, _)| event.clone())
                        .collect()
                };

                let txn_output = vm_output
                    .into_transaction_output_with_materialized_write_set(
                        Vec::new(),
                        patched_resource_write_set,
                        patched_events,
                    )
                    .map_err(|err| {
                        VMStatus::error(
                            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                            Some(err.to_string()),
                        )
                    })?;
                Ok((txn_idx, txn_output))
            })
            .collect::<Result<Vec<_>, VMStatus>>()?;

        results.sort_by_key(|(idx, _)| *idx);
        return Ok(results.into_iter().map(|(_, output)| output).collect());
    }

    let mut state_cache = StateViewCache::new(state_view);
    let mut group_cache: HashMap<StateKey, BTreeMap<StructTag, Bytes>> = HashMap::new();
    let mut results = Vec::with_capacity(outputs.len());

    for (txn_idx, output) in outputs.into_iter() {
        let (mut vm_output, group_read_layouts) = output.into_inner();
        let has_delayed = vm_output.contains_delayed_fields();
        let has_group_ops = vm_output.resource_write_set().values().any(|op| {
            matches!(
                op,
                AbstractResourceWriteOp::WriteResourceGroup(_)
                    | AbstractResourceWriteOp::ResourceGroupInPlaceDelayedFieldChange(_)
            )
        });
        if !has_delayed && vm_output.aggregator_v1_delta_set().is_empty() && !has_group_ops {
            let txn_output = vm_output.into_transaction_output().map_err(|err| {
                VMStatus::error(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                    Some(err.to_string()),
                )
            })?;
            state_cache
                .push_write_set(txn_output.write_set())
                .map_err(|err| {
                    VMStatus::error(
                        StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
                        Some(format!("Failed to apply write set: {:?}", err)),
                    )
                })?;
            results.push(txn_output);
            continue;
        }

        if has_delayed {
            vm_output.try_materialize(&state_cache)?;
        }
        let mapping = DelayedFieldValueMapping {
            delayed_fields: &delayed_fields,
            txn_idx,
        };

        let patched_resource_write_set = materialize_resource_write_set(
            &vm_output,
            &mapping,
            &delayed_field_cache,
            &group_read_layouts,
            &state_cache,
            &mut group_cache,
            has_delayed,
        )?;
        let patched_events = if has_delayed {
            materialize_events(&vm_output, &mapping)?
        } else {
            vm_output
                .events()
                .iter()
                .map(|(event, _)| event.clone())
                .collect()
        };

        let txn_output = vm_output
            .into_transaction_output_with_materialized_write_set(
                Vec::new(),
                patched_resource_write_set,
                patched_events,
            )
            .map_err(|err| {
                VMStatus::error(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                    Some(err.to_string()),
                )
            })?;

        state_cache
            .push_write_set(txn_output.write_set())
            .map_err(|err| {
                VMStatus::error(
                    StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
                    Some(format!("Failed to apply write set: {:?}", err)),
                )
            })?;

        results.push(txn_output);
    }

    Ok(results)
}

fn materialize_resource_write_set_no_groups(
    output: &VMOutput,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    delayed_field_cache: &DelayedFieldCache,
) -> Result<Vec<(StateKey, WriteOp)>, VMStatus> {
    let mut patched = Vec::new();

    for (key, op) in output.resource_write_set() {
        let write_op = match op {
            AbstractResourceWriteOp::Write(_write_op) => continue,
            AbstractResourceWriteOp::WriteWithDelayedFields(WriteWithDelayedFieldsOp {
                write_op,
                layout,
                ..
            }) => {
                delayed_field_cache.insert_base_value(key.clone(), write_op.clone(), true);
                materialize_write_op_with_layout(write_op, layout.as_ref(), mapping)?
            }
            AbstractResourceWriteOp::InPlaceDelayedFieldChange(InPlaceDelayedFieldChangeOp {
                layout,
                metadata,
                ..
            }) => materialize_in_place_change(
                key,
                layout.as_ref(),
                metadata,
                mapping,
                delayed_field_cache,
            )?,
            AbstractResourceWriteOp::WriteResourceGroup(_)
            | AbstractResourceWriteOp::ResourceGroupInPlaceDelayedFieldChange(_) => {
                return Err(VMStatus::error(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                    Some(
                        "unexpected resource group write in fast materialization path".to_string(),
                    ),
                ));
            }
        };

        patched.push((key.clone(), write_op));
    }

    Ok(patched)
}

pub(crate) fn materialize_resource_write_set<S: StateView>(
    output: &VMOutput,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    delayed_field_cache: &DelayedFieldCache,
    group_read_layouts: &HashMap<StateKey, BTreeMap<StructTag, Arc<MoveTypeLayout>>>,
    state_view: &S,
    group_cache: &mut HashMap<StateKey, BTreeMap<StructTag, Bytes>>,
    materialize_delayed: bool,
) -> Result<Vec<(StateKey, WriteOp)>, VMStatus> {
    let mut patched = Vec::new();

    for (key, op) in output.resource_write_set() {
        let write_op = match op {
            // Already materialized; nothing to patch.
            AbstractResourceWriteOp::Write(_write_op) => continue,
            AbstractResourceWriteOp::WriteWithDelayedFields(WriteWithDelayedFieldsOp {
                write_op,
                layout,
                ..
            }) => {
                if materialize_delayed {
                    delayed_field_cache.insert_base_value(key.clone(), write_op.clone(), true);
                    materialize_write_op_with_layout(write_op, layout.as_ref(), mapping)?
                } else {
                    return Err(VMStatus::error(
                        StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                        Some(
                            "unexpected WriteWithDelayedFields without delayed fields".to_string(),
                        ),
                    ));
                }
            }
            AbstractResourceWriteOp::InPlaceDelayedFieldChange(InPlaceDelayedFieldChangeOp {
                layout,
                metadata,
                ..
            }) => {
                if !materialize_delayed {
                    return Err(VMStatus::error(
                        StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                        Some("unexpected delayed field change without delayed fields".to_string()),
                    ));
                }
                materialize_in_place_change(
                    key,
                    layout.as_ref(),
                    metadata,
                    mapping,
                    delayed_field_cache,
                )?
            }
            AbstractResourceWriteOp::WriteResourceGroup(group_write) => {
                if materialize_delayed {
                    for (tag, (inner_op, _)) in group_write.inner_ops() {
                        match inner_op {
                            WriteOp::Creation { data, .. } | WriteOp::Modification { data, .. } => {
                                delayed_field_cache.insert_group_member_value(
                                    key.clone(),
                                    tag.clone(),
                                    data.clone(),
                                );
                            }
                            WriteOp::Deletion { .. } => {
                                delayed_field_cache.remove_group_member_value(key, tag);
                            }
                        }
                    }
                }
                materialize_group_write(
                    key,
                    group_write,
                    mapping,
                    state_view,
                    group_cache,
                    materialize_delayed,
                )?
            }
            AbstractResourceWriteOp::ResourceGroupInPlaceDelayedFieldChange(
                ResourceGroupInPlaceDelayedFieldChangeOp { metadata, .. },
            ) => {
                if !materialize_delayed {
                    return Err(VMStatus::error(
                        StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                        Some(
                            "unexpected group delayed field change without delayed fields"
                                .to_string(),
                        ),
                    ));
                }
                materialize_group_in_place(
                    key,
                    metadata,
                    mapping,
                    delayed_field_cache,
                    group_read_layouts,
                    state_view,
                    group_cache,
                )?
            }
        };

        patched.push((key.clone(), write_op));
    }

    Ok(patched)
}

fn materialize_write_op_with_layout(
    write_op: &WriteOp,
    layout: &MoveTypeLayout,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
) -> Result<WriteOp, VMStatus> {
    match write_op {
        WriteOp::Deletion { metadata } => Ok(WriteOp::Deletion {
            metadata: metadata.clone(),
        }),
        WriteOp::Creation { data, metadata } => {
            let bytes = materialize_bytes_force(data, layout, mapping)?;
            Ok(WriteOp::Creation {
                data: bytes,
                metadata: metadata.clone(),
            })
        }
        WriteOp::Modification { data, metadata } => {
            let bytes = materialize_bytes_force(data, layout, mapping)?;
            Ok(WriteOp::Modification {
                data: bytes,
                metadata: metadata.clone(),
            })
        }
    }
}

fn materialize_in_place_change(
    key: &StateKey,
    layout: &MoveTypeLayout,
    metadata: &starcoin_vm_types::state_store::state_value::StateValueMetadata,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    delayed_field_cache: &DelayedFieldCache,
) -> Result<WriteOp, VMStatus> {
    let base = delayed_field_cache.get_base_value(key).ok_or_else(|| {
        VMStatus::error(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            Some(format!(
                "Missing cached base value for delayed field exchange: {:?}",
                key
            )),
        )
    })?;
    let bytes = base
        .bytes()
        .ok_or_else(|| {
            VMStatus::error(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                Some(format!("Missing bytes for cached base value: {:?}", key)),
            )
        })?
        .clone();
    let materialized = materialize_bytes_force(&bytes, layout, mapping)?;
    Ok(WriteOp::Modification {
        data: materialized,
        metadata: metadata.clone(),
    })
}

fn materialize_group_write<S: StateView>(
    key: &StateKey,
    group_write: &GroupWrite,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    state_view: &S,
    group_cache: &mut HashMap<StateKey, BTreeMap<StructTag, Bytes>>,
    materialize_delayed: bool,
) -> Result<WriteOp, VMStatus> {
    let mut remove_cache = false;
    let group_map = load_group_map_cached(state_view, group_cache, key)?;
    for (tag, inner) in group_write.inner_ops() {
        let (inner_op, layout) = inner;
        match inner_op {
            WriteOp::Deletion { .. } => {
                group_map.remove(tag);
            }
            WriteOp::Creation { data, .. } | WriteOp::Modification { data, .. } => {
                let bytes = if materialize_delayed {
                    if let Some(layout) = layout.as_ref() {
                        materialize_bytes(data, layout.as_ref(), mapping)?
                    } else {
                        data.clone()
                    }
                } else {
                    data.clone()
                };
                group_map.insert(tag.clone(), bytes);
            }
        }
    }

    let metadata_op = group_write.metadata_op();
    let op = match metadata_op {
        WriteOp::Deletion { metadata } => {
            group_map.clear();
            remove_cache = true;
            WriteOp::Deletion {
                metadata: metadata.clone(),
            }
        }
        WriteOp::Creation { metadata, .. } => WriteOp::Creation {
            data: serialize_group_map(group_map)?,
            metadata: metadata.clone(),
        },
        WriteOp::Modification { metadata, .. } => WriteOp::Modification {
            data: serialize_group_map(group_map)?,
            metadata: metadata.clone(),
        },
    };
    if remove_cache {
        group_cache.remove(key);
    }
    Ok(op)
}

fn materialize_group_in_place<S: StateView>(
    key: &StateKey,
    metadata: &starcoin_vm_types::state_store::state_value::StateValueMetadata,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    delayed_field_cache: &DelayedFieldCache,
    group_read_layouts: &HashMap<StateKey, BTreeMap<StructTag, Arc<MoveTypeLayout>>>,
    state_view: &S,
    group_cache: &mut HashMap<StateKey, BTreeMap<StructTag, Bytes>>,
) -> Result<WriteOp, VMStatus> {
    let group_map = load_group_map_cached(state_view, group_cache, key)?;
    let layouts = group_read_layouts.get(key).ok_or_else(|| {
        VMStatus::error(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            Some(format!(
                "Missing group read layouts for delayed field exchange: {:?}",
                key
            )),
        )
    })?;

    for (tag, layout) in layouts {
        let cached = delayed_field_cache
            .get_group_member_value(key, tag)
            .ok_or_else(|| {
                VMStatus::error(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                    Some(format!(
                        "Missing cached group member value for {:?}::{:?}",
                        key, tag
                    )),
                )
            })?;
        let bytes = materialize_bytes_force(&cached, layout.as_ref(), mapping)?;
        group_map.insert(tag.clone(), bytes);
    }

    Ok(WriteOp::Modification {
        data: serialize_group_map(group_map)?,
        metadata: metadata.clone(),
    })
}

pub(crate) fn materialize_events(
    output: &VMOutput,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
) -> Result<Vec<ContractEvent>, VMStatus> {
    output
        .events()
        .iter()
        .map(|(event, layout)| match layout {
            None => Ok(event.clone()),
            Some(layout) => materialize_event(event, layout, mapping),
        })
        .collect()
}

fn materialize_event(
    event: &ContractEvent,
    layout: &MoveTypeLayout,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
) -> Result<ContractEvent, VMStatus> {
    let data = materialize_bytes(&Bytes::copy_from_slice(event.event_data()), layout, mapping)?;
    Ok(match event {
        ContractEvent::V1(event_v1) => ContractEvent::new_v1(
            *event_v1.key(),
            event_v1.sequence_number(),
            event_v1.type_tag().clone(),
            data.to_vec(),
        ),
        ContractEvent::V2(event_v2) => {
            ContractEvent::new_v2(event_v2.type_tag().clone(), data.to_vec())
        }
    })
}

fn materialize_bytes(
    bytes: &Bytes,
    layout: &MoveTypeLayout,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
) -> Result<Bytes, VMStatus> {
    let value = deserialize_and_allow_delayed_values(bytes, layout).ok_or_else(|| {
        VMStatus::error(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            Some("Failed to deserialize value with delayed fields".to_string()),
        )
    })?;
    let mut ids: HashSet<u64> = HashSet::new();
    find_identifiers_in_value(&value, &mut ids).map_err(|err| {
        VMStatus::error(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            Some(format!(
                "Failed to scan delayed field identifiers: {:?}",
                err
            )),
        )
    })?;
    if ids.is_empty() {
        return Ok(bytes.clone());
    }
    materialize_bytes_force_with_value(value, layout, mapping)
}

fn materialize_bytes_force(
    bytes: &Bytes,
    layout: &MoveTypeLayout,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
) -> Result<Bytes, VMStatus> {
    let value = deserialize_and_allow_delayed_values(bytes, layout).ok_or_else(|| {
        VMStatus::error(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            Some("Failed to deserialize value with delayed fields".to_string()),
        )
    })?;
    materialize_bytes_force_with_value(value, layout, mapping)
}

fn materialize_bytes_force_with_value(
    value: move_vm_types::values::Value,
    layout: &MoveTypeLayout,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
) -> Result<Bytes, VMStatus> {
    let serialized =
        serialize_and_replace_ids_with_values(&value, layout, mapping).ok_or_else(|| {
            VMStatus::error(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                Some("Failed to serialize value with delayed fields".to_string()),
            )
        })?;
    Ok(Bytes::from(serialized))
}

fn load_group_map<S: StateView>(
    state_view: &S,
    key: &StateKey,
) -> Result<BTreeMap<StructTag, Bytes>, VMStatus> {
    let maybe_state = state_view.get_state_value(key).map_err(|err| {
        VMStatus::error(
            StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            Some(format!("Failed to read group state {:?}: {:?}", key, err)),
        )
    })?;
    let bytes = maybe_state.map(|v| v.bytes().clone());
    match bytes {
        None => Ok(BTreeMap::new()),
        Some(bytes) if bytes.is_empty() => Ok(BTreeMap::new()),
        Some(bytes) => bcs::from_bytes(&bytes).map_err(|err| {
            VMStatus::error(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                Some(format!("Failed to decode group bytes {:?}: {:?}", key, err)),
            )
        }),
    }
}

fn load_group_map_cached<'a, S: StateView>(
    state_view: &'a S,
    cache: &'a mut HashMap<StateKey, BTreeMap<StructTag, Bytes>>,
    key: &'a StateKey,
) -> Result<&'a mut BTreeMap<StructTag, Bytes>, VMStatus> {
    if !cache.contains_key(key) {
        let map = load_group_map(state_view, key)?;
        cache.insert(key.clone(), map);
    }
    Ok(cache
        .get_mut(key)
        .expect("cache entry should be present after load"))
}

fn serialize_group_map(group_map: &BTreeMap<StructTag, Bytes>) -> Result<Bytes, VMStatus> {
    bcs::to_bytes(group_map).map(Bytes::from).map_err(|err| {
        VMStatus::error(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            Some(format!("Failed to encode group bytes: {:?}", err)),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use move_core_types::account_address::AccountAddress;
    use move_core_types::identifier::Identifier;
    use move_core_types::language_storage::StructTag;
    use move_core_types::value::{MoveStructLayout, MoveTypeLayout};
    use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
    use move_vm_types::value_serde::serialize_and_allow_delayed_values;
    use move_vm_types::values::{Struct, Value};
    use starcoin_vm_types::state_store::state_value::StateValueMetadata;
    use starcoin_vm_types::write_set::WriteOp;

    struct TestMapping {
        value: u64,
    }

    impl ValueToIdentifierMapping for TestMapping {
        type Identifier = DelayedFieldID;

        fn value_to_identifier(
            &self,
            _kind: &move_core_types::value::IdentifierMappingKind,
            _layout: &MoveTypeLayout,
            _value: move_vm_types::values::Value,
        ) -> Result<Self::Identifier, PartialVMError> {
            Err(PartialVMError::new(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            ))
        }

        fn identifier_to_value(
            &self,
            _layout: &MoveTypeLayout,
            _identifier: Self::Identifier,
        ) -> Result<move_vm_types::values::Value, PartialVMError> {
            Ok(move_vm_types::values::Value::u64(self.value))
        }
    }

    #[test]
    fn materialize_in_place_uses_cached_base_with_ids() {
        let address = AccountAddress::from_hex_literal("0x1").unwrap();
        let struct_tag = StructTag {
            address,
            module: Identifier::new("Test").unwrap(),
            name: Identifier::new("Resource").unwrap(),
            type_args: vec![],
        };
        let state_key = StateKey::resource(&address, &struct_tag).unwrap();
        let layout_delayed = MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Native(
                move_core_types::value::IdentifierMappingKind::Aggregator,
                Box::new(MoveTypeLayout::U64),
            ),
        ]));
        let layout_plain = MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::U64,
        ]));
        let delayed_id = DelayedFieldID::new_with_width(7, 8);

        let _base_value = Value::struct_(Struct::pack(vec![
            Value::u64(1),
            Value::delayed_value(delayed_id),
        ]));
        let updated_value = Value::struct_(Struct::pack(vec![
            Value::u64(99),
            Value::delayed_value(delayed_id),
        ]));
        let updated_bytes = serialize_and_allow_delayed_values(&updated_value, &layout_delayed)
            .unwrap()
            .unwrap();

        let delayed_field_cache = DelayedFieldCache::default();
        delayed_field_cache.insert_base_value(
            state_key.clone(),
            WriteOp::Modification {
                data: Bytes::from(updated_bytes),
                metadata: StateValueMetadata::none(),
            },
            true,
        );

        let mapping = TestMapping { value: 555 };
        let output = materialize_in_place_change(
            &state_key,
            &layout_delayed,
            &StateValueMetadata::none(),
            &mapping,
            &delayed_field_cache,
        )
        .unwrap();
        let bytes = output.bytes().unwrap().clone();
        let value = Value::simple_deserialize(&bytes, &layout_plain).unwrap();
        let mut fields = value.value_as::<Struct>().unwrap().unpack().unwrap();
        let field0 = fields.next().unwrap().value_as::<u64>().unwrap();
        let field1 = fields.next().unwrap().value_as::<u64>().unwrap();

        assert_eq!(field0, 99);
        assert_eq!(field1, 555);
    }
}
