// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod storage_wrapper;
mod vm_wrapper;

use crate::{
    data_cache::{effective_max_value_nest_depth, take_resource_group_stats, StateViewCache},
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
#[cfg(test)]
use move_vm_runtime::config::DEFAULT_MAX_VALUE_NEST_DEPTH;
use move_vm_types::delayed_values::delayed_field_id::{DelayedFieldID, ExtractWidth};
use move_vm_types::value_serde::{ValueSerDeContext, ValueToIdentifierMapping};
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
    write_set::{TransactionWrite, WriteOp},
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
        let max_value_nest_depth = effective_max_value_nest_depth(state_view);
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
            (
                state_view,
                delayed_field_cache.clone(),
                max_value_nest_depth,
            ),
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
                    max_value_nest_depth,
                )?;
                let materialize_ms = materialize_start.elapsed().as_secs_f64() * 1000.0;
                let rg_stats = take_resource_group_stats();
                info!(
                    target: "vm-bench",
                    "parallel execute done: exec_ms={:.3} materialize_ms={:.3} rg_accesses={} rg_cache_hits={} rg_member_calls={} rg_member_ms={:.3} rg_size_calls={} rg_size_ms={:.3}",
                    exec_ms,
                    materialize_ms,
                    rg_stats.group_accesses,
                    rg_stats.group_cache_hits,
                    rg_stats.group_member_calls,
                    rg_stats.group_member_ns as f64 / 1_000_000.0,
                    rg_stats.group_size_calls,
                    rg_stats.group_size_ns as f64 / 1_000_000.0,
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
    ) -> Result<DelayedFieldID, PartialVMError> {
        Err(PartialVMError::new(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
        ))
    }

    fn identifier_to_value(
        &self,
        layout: &MoveTypeLayout,
        identifier: DelayedFieldID,
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
    max_value_nest_depth: Option<u64>,
) -> Result<Vec<TransactionOutput>, VMStatus> {
    fn is_group_write_op(op: &AbstractResourceWriteOp) -> bool {
        matches!(
            op,
            AbstractResourceWriteOp::WriteResourceGroup(_)
                | AbstractResourceWriteOp::ResourceGroupInPlaceDelayedFieldChange(_)
        )
    }

    fn materialize_parallel_candidate<S: StateView + Sync>(
        txn_idx: usize,
        output: StarcoinTransactionOutput,
        delayed_fields: &VersionedDelayedFields<DelayedFieldID>,
        delayed_field_cache: &DelayedFieldCache,
        state_view: &S,
        max_value_nest_depth: Option<u64>,
    ) -> Result<(usize, TransactionOutput), VMStatus> {
        let (vm_output, group_read_layouts) = output.into_inner();
        let has_delayed = vm_output.contains_delayed_fields();
        let has_group_ops = vm_output
            .resource_write_set()
            .values()
            .any(is_group_write_op);
        if !has_delayed && vm_output.aggregator_v1_delta_set().is_empty() && !has_group_ops {
            let txn_output = vm_output.into_transaction_output().map_err(|err| {
                VMStatus::error(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                    Some(err.to_string()),
                )
            })?;
            return Ok((txn_idx, txn_output));
        }

        let mapping = DelayedFieldValueMapping {
            delayed_fields,
            txn_idx,
        };

        let mut group_cache = HashMap::new();
        let patched_resource_write_set = if has_group_ops {
            materialize_resource_write_set(
                &vm_output,
                &mapping,
                delayed_field_cache,
                &group_read_layouts,
                state_view,
                &mut group_cache,
                has_delayed,
                max_value_nest_depth,
            )?
        } else {
            materialize_resource_write_set_no_groups(
                &vm_output,
                &mapping,
                delayed_field_cache,
                state_view,
                max_value_nest_depth,
            )?
        };
        let patched_events = if has_delayed {
            materialize_events(&vm_output, &mapping, max_value_nest_depth)?
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
    }

    let mut outputs = outputs;
    let mut has_agg_v1 = false;
    let mut group_touches = 0u64;
    let mut group_touch_counts: HashMap<StateKey, usize> = HashMap::new();
    let mut delayed_resource_touch_counts: HashMap<StateKey, usize> = HashMap::new();
    for (_, output) in outputs.iter() {
        if !output.output.aggregator_v1_delta_set().is_empty() {
            has_agg_v1 = true;
        }
        for (key, op) in output.output.resource_write_set() {
            if matches!(
                op,
                AbstractResourceWriteOp::WriteWithDelayedFields(_)
                    | AbstractResourceWriteOp::InPlaceDelayedFieldChange(_)
            ) {
                *delayed_resource_touch_counts
                    .entry(key.clone())
                    .or_insert(0) += 1;
            }
            if is_group_write_op(op) {
                group_touches += 1;
                *group_touch_counts.entry(key.clone()).or_insert(0) += 1;
            }
        }
    }
    let has_group_dup = group_touch_counts.values().any(|count| *count > 1);
    let has_delayed_resource_dup = delayed_resource_touch_counts
        .values()
        .any(|count| *count > 1);
    let needs_sequential = has_agg_v1 || has_group_dup || has_delayed_resource_dup;
    outputs.sort_by_key(|(idx, _)| *idx);

    if !needs_sequential {
        let mut results = outputs
            .into_par_iter()
            .map(|(txn_idx, output)| {
                materialize_parallel_candidate(
                    txn_idx,
                    output,
                    &delayed_fields,
                    delayed_field_cache.as_ref(),
                    state_view,
                    max_value_nest_depth,
                )
            })
            .collect::<Result<Vec<_>, VMStatus>>()?;

        results.sort_by_key(|(idx, _)| *idx);
        return Ok(results.into_iter().map(|(_, output)| output).collect());
    }

    info!(
        target: "vm-bench",
        "materialize sequential: agg_v1={} group_dup={} delayed_resource_dup={} group_touches={}",
        has_agg_v1,
        has_group_dup,
        has_delayed_resource_dup,
        group_touches
    );
    let mut state_cache = StateViewCache::new(state_view);
    let mut group_cache: HashMap<StateKey, BTreeMap<StructTag, Bytes>> = HashMap::new();
    let ordered_indices = outputs.iter().map(|(idx, _)| *idx).collect::<Vec<_>>();
    let mut sequential_outputs = HashMap::new();
    let mut parallel_candidates = Vec::new();
    for (txn_idx, output) in outputs.into_iter() {
        let has_delayed = output.output.contains_delayed_fields();
        let has_agg_v1 = !output.output.aggregator_v1_delta_set().is_empty();
        let touches_duplicated_group =
            output.output.resource_write_set().iter().any(|(key, op)| {
                is_group_write_op(op) && group_touch_counts.get(key).copied().unwrap_or(0) > 1
            });
        // In mixed mode, keep delayed-field outputs in sequential path so
        // delayed_field_cache updates remain in transaction order.
        if has_delayed || has_agg_v1 || touches_duplicated_group {
            sequential_outputs.insert(txn_idx, output);
        } else {
            parallel_candidates.push((txn_idx, output));
        }
    }

    let mut parallel_results = parallel_candidates
        .into_par_iter()
        .map(|(txn_idx, output)| {
            materialize_parallel_candidate(
                txn_idx,
                output,
                &delayed_fields,
                delayed_field_cache.as_ref(),
                state_view,
                max_value_nest_depth,
            )
        })
        .collect::<Result<Vec<_>, VMStatus>>()?
        .into_iter()
        .collect::<HashMap<_, _>>();

    let mut results = Vec::with_capacity(ordered_indices.len());
    for txn_idx in ordered_indices {
        let txn_output = if let Some(output) = sequential_outputs.remove(&txn_idx) {
            let (mut vm_output, group_read_layouts) = output.into_inner();
            let has_delayed = vm_output.contains_delayed_fields();
            let has_agg_v1 = !vm_output.aggregator_v1_delta_set().is_empty();
            let has_group_ops = vm_output
                .resource_write_set()
                .values()
                .any(is_group_write_op);
            if !has_delayed && vm_output.aggregator_v1_delta_set().is_empty() && !has_group_ops {
                vm_output.into_transaction_output().map_err(|err| {
                    VMStatus::error(
                        StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                        Some(err.to_string()),
                    )
                })?
            } else {
                if has_delayed || has_agg_v1 {
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
                    max_value_nest_depth,
                )?;
                let patched_events = if has_delayed {
                    materialize_events(&vm_output, &mapping, max_value_nest_depth)?
                } else {
                    vm_output
                        .events()
                        .iter()
                        .map(|(event, _)| event.clone())
                        .collect()
                };

                vm_output
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
                    })?
            }
        } else {
            parallel_results.remove(&txn_idx).ok_or_else(|| {
                VMStatus::error(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                    Some(format!("Missing materialized output for txn {}", txn_idx)),
                )
            })?
        };

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

    if !sequential_outputs.is_empty() || !parallel_results.is_empty() {
        return Err(VMStatus::error(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            Some(format!(
                "Unconsumed outputs after mixed materialization: seq={}, par={}",
                sequential_outputs.len(),
                parallel_results.len()
            )),
        ));
    }

    Ok(results)
}

fn materialize_resource_write_set_no_groups(
    output: &VMOutput,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    delayed_field_cache: &DelayedFieldCache,
    state_view: &impl StateView,
    max_value_nest_depth: Option<u64>,
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
                materialize_write_op_with_layout(
                    write_op,
                    layout.as_ref(),
                    mapping,
                    max_value_nest_depth,
                )?
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
                state_view,
                max_value_nest_depth,
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
    max_value_nest_depth: Option<u64>,
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
                    materialize_write_op_with_layout(
                        write_op,
                        layout.as_ref(),
                        mapping,
                        max_value_nest_depth,
                    )?
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
                    state_view,
                    max_value_nest_depth,
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
                    max_value_nest_depth,
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
                    max_value_nest_depth,
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
    max_value_nest_depth: Option<u64>,
) -> Result<WriteOp, VMStatus> {
    match write_op {
        WriteOp::Deletion { metadata } => Ok(WriteOp::Deletion {
            metadata: metadata.clone(),
        }),
        WriteOp::Creation { data, metadata } => {
            let bytes = materialize_bytes_force(data, layout, mapping, max_value_nest_depth)?;
            Ok(WriteOp::Creation {
                data: bytes,
                metadata: metadata.clone(),
            })
        }
        WriteOp::Modification { data, metadata } => {
            let bytes = materialize_bytes_force(data, layout, mapping, max_value_nest_depth)?;
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
    state_view: &impl StateView,
    max_value_nest_depth: Option<u64>,
) -> Result<WriteOp, VMStatus> {
    let base = if let Some(cached_base) = delayed_field_cache.get_base_value(key) {
        cached_base
    } else {
        let state_value = state_view
            .get_state_value(key)
            .map_err(|err| {
                VMStatus::error(
                    StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
                    Some(format!(
                        "Failed to read base state value for delayed field exchange {:?}: {:?}",
                        key, err
                    )),
                )
            })?
            .ok_or_else(|| {
                VMStatus::error(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                    Some(format!(
                        "Missing base state value for delayed field exchange: {:?}",
                        key
                    )),
                )
            })?;

        let base_from_state = WriteOp::from_state_value(Some(state_value));
        delayed_field_cache.insert_base_value(key.clone(), base_from_state.clone(), true);
        base_from_state
    };
    let bytes = base
        .bytes()
        .ok_or_else(|| {
            VMStatus::error(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                Some(format!("Missing bytes for cached base value: {:?}", key)),
            )
        })?
        .clone();
    let materialized = materialize_bytes_force(&bytes, layout, mapping, max_value_nest_depth)?;
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
    max_value_nest_depth: Option<u64>,
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
                        materialize_bytes(data, layout.as_ref(), mapping, max_value_nest_depth)?
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
    max_value_nest_depth: Option<u64>,
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
        let bytes =
            materialize_bytes_force(&cached, layout.as_ref(), mapping, max_value_nest_depth)?;
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
    max_value_nest_depth: Option<u64>,
) -> Result<Vec<ContractEvent>, VMStatus> {
    output
        .events()
        .iter()
        .map(|(event, layout)| match layout {
            None => Ok(event.clone()),
            Some(layout) => materialize_event(event, layout, mapping, max_value_nest_depth),
        })
        .collect()
}

fn materialize_event(
    event: &ContractEvent,
    layout: &MoveTypeLayout,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    max_value_nest_depth: Option<u64>,
) -> Result<ContractEvent, VMStatus> {
    let data = materialize_bytes(
        &Bytes::copy_from_slice(event.event_data()),
        layout,
        mapping,
        max_value_nest_depth,
    )?;
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
    max_value_nest_depth: Option<u64>,
) -> Result<Bytes, VMStatus> {
    let value = ValueSerDeContext::<DelayedFieldID>::new(max_value_nest_depth)
        .with_delayed_fields_serde()
        .deserialize(bytes, layout)
        .ok_or_else(|| {
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
    materialize_bytes_force_with_value(value, layout, mapping, max_value_nest_depth)
}

fn materialize_bytes_force(
    bytes: &Bytes,
    layout: &MoveTypeLayout,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    max_value_nest_depth: Option<u64>,
) -> Result<Bytes, VMStatus> {
    let value = ValueSerDeContext::<DelayedFieldID>::new(max_value_nest_depth)
        .with_delayed_fields_serde()
        .deserialize(bytes, layout)
        .ok_or_else(|| {
            VMStatus::error(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                Some("Failed to deserialize value with delayed fields".to_string()),
            )
        })?;
    materialize_bytes_force_with_value(value, layout, mapping, max_value_nest_depth)
}

fn materialize_bytes_force_with_value(
    value: move_vm_types::values::Value,
    layout: &MoveTypeLayout,
    mapping: &impl ValueToIdentifierMapping<Identifier = DelayedFieldID>,
    max_value_nest_depth: Option<u64>,
) -> Result<Bytes, VMStatus> {
    let serialized = ValueSerDeContext::<DelayedFieldID>::new(max_value_nest_depth)
        .with_delayed_fields_replacement(mapping)
        .serialize(&value, layout)
        .map_err(|err| {
            VMStatus::error(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                Some(format!(
                    "Failed to serialize value with delayed fields replacement: {}",
                    err
                )),
            )
        })?
        .ok_or_else(|| {
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
    use move_core_types::vm_status::KeptVMStatus;
    use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
    use move_vm_types::value_serde::ValueSerDeContext;
    use move_vm_types::values::{Struct, Value};
    use starcoin_aggregator::bounded_math::SignedU128;
    use starcoin_aggregator::delta_change_set::DeltaOp;
    use starcoin_aggregator::delta_math::DeltaHistory;
    use starcoin_aggregator::types::DelayedFieldValue;
    use starcoin_vm_runtime_types::change_set::VMChangeSet;
    use starcoin_vm_runtime_types::module_write_set::ModuleWriteSet;
    use starcoin_vm_runtime_types::resolver::ResourceGroupSize;
    use starcoin_vm_types::fee_statement::FeeStatement;
    use starcoin_vm_types::state_store::in_memory_state_view::InMemoryStateView;
    use starcoin_vm_types::state_store::state_value::StateValue;
    use starcoin_vm_types::state_store::state_value::StateValueMetadata;
    use starcoin_vm_types::transaction::TransactionAuxiliaryData;
    use starcoin_vm_types::write_set::WriteOp;
    use std::time::{Duration, Instant};

    mod tests_delayed_from_state;

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
        ) -> Result<DelayedFieldID, PartialVMError> {
            Err(PartialVMError::new(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
            ))
        }

        fn identifier_to_value(
            &self,
            _layout: &MoveTypeLayout,
            _identifier: DelayedFieldID,
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
        let updated_bytes =
            ValueSerDeContext::<DelayedFieldID>::new(Some(DEFAULT_MAX_VALUE_NEST_DEPTH))
                .with_delayed_fields_serde()
                .serialize(&updated_value, &layout_delayed)
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
            &InMemoryStateView::new(HashMap::new()),
            Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
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

    fn build_sparse_conflict_case(
        txn_count: usize,
        conflict_a: usize,
        conflict_b: usize,
        members_per_group: usize,
        bytes_per_member: usize,
    ) -> (
        Vec<(usize, StarcoinTransactionOutput)>,
        InMemoryStateView,
        StateKey,
    ) {
        assert!(txn_count > conflict_a);
        assert!(txn_count > conflict_b);
        assert_ne!(conflict_a, conflict_b);

        let mut outputs = Vec::with_capacity(txn_count);
        let agg_key = StateKey::raw(b"bench-agg");

        for txn_idx in 0..txn_count {
            if txn_idx == conflict_a || txn_idx == conflict_b {
                let mut history = DeltaHistory::new();
                history.record_success(SignedU128::Positive(1));
                let mut agg_delta_set = BTreeMap::new();
                agg_delta_set.insert(
                    agg_key.clone(),
                    DeltaOp::new(SignedU128::Positive(1), 1_000_000_000, history),
                );
                let vm_output = VMOutput::new(
                    VMChangeSet::new(
                        BTreeMap::new(),
                        vec![],
                        BTreeMap::new(),
                        BTreeMap::new(),
                        agg_delta_set,
                    ),
                    ModuleWriteSet::empty(),
                    FeeStatement::zero(),
                    TransactionStatus::Keep(KeptVMStatus::Executed),
                    TransactionAuxiliaryData::None,
                );
                outputs.push((
                    txn_idx,
                    StarcoinTransactionOutput::new(vm_output, HashMap::new()),
                ));
                continue;
            }

            let group_key = StateKey::raw(format!("group-{txn_idx}").as_bytes());
            let mut inner_ops = BTreeMap::new();
            let mut expected_group_map = BTreeMap::new();
            for member_idx in 0..members_per_group {
                let tag = StructTag {
                    address: AccountAddress::ONE,
                    module: Identifier::new(format!("M{member_idx}")).unwrap(),
                    name: Identifier::new(format!("N{member_idx}")).unwrap(),
                    type_args: vec![],
                };
                let fill = ((txn_idx + member_idx) % 251) as u8;
                let bytes = Bytes::from(vec![fill; bytes_per_member]);
                expected_group_map.insert(tag.clone(), bytes.clone());
                inner_ops.insert(tag, (WriteOp::legacy_modification(bytes), None));
            }
            let group_size = ResourceGroupSize::Concrete(
                serialize_group_map(&expected_group_map).unwrap().len() as u64,
            );
            let group_write = GroupWrite::new(
                WriteOp::legacy_modification(Bytes::new()),
                inner_ops,
                group_size,
                0,
            );

            let mut resource_write_set = BTreeMap::new();
            resource_write_set.insert(
                group_key,
                AbstractResourceWriteOp::WriteResourceGroup(group_write),
            );
            let vm_output = VMOutput::new(
                VMChangeSet::new(
                    resource_write_set,
                    vec![],
                    BTreeMap::new(),
                    BTreeMap::new(),
                    BTreeMap::new(),
                ),
                ModuleWriteSet::empty(),
                FeeStatement::zero(),
                TransactionStatus::Keep(KeptVMStatus::Executed),
                TransactionAuxiliaryData::None,
            );
            outputs.push((
                txn_idx,
                StarcoinTransactionOutput::new(vm_output, HashMap::new()),
            ));
        }

        let mut state_data = HashMap::new();
        state_data.insert(
            agg_key.clone(),
            StateValue::new_legacy(Bytes::from(bcs::to_bytes(&100u128).unwrap())),
        );

        (outputs, InMemoryStateView::new(state_data), agg_key)
    }

    fn build_delayed_only_dependency_case(
        consumer_count: usize,
    ) -> (
        Vec<(usize, StarcoinTransactionOutput)>,
        InMemoryStateView,
        VersionedDelayedFields<DelayedFieldID>,
        StateKey,
    ) {
        let address = AccountAddress::from_hex_literal("0x1").unwrap();
        let struct_tag = StructTag {
            address,
            module: Identifier::new("DelayedOnly").unwrap(),
            name: Identifier::new("Resource").unwrap(),
            type_args: vec![],
        };
        let state_key = StateKey::resource(&address, &struct_tag).unwrap();
        let layout = Arc::new(MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![
            MoveTypeLayout::U64,
            MoveTypeLayout::Native(
                move_core_types::value::IdentifierMappingKind::Aggregator,
                Box::new(MoveTypeLayout::U64),
            ),
        ])));
        let delayed_id = DelayedFieldID::new_with_width(101, 8);
        let delayed_value = Value::struct_(Struct::pack(vec![
            Value::u64(1),
            Value::delayed_value(delayed_id),
        ]));
        let delayed_bytes = ValueSerDeContext::new(Some(DEFAULT_MAX_VALUE_NEST_DEPTH))
            .with_delayed_fields_serde()
            .serialize(&delayed_value, layout.as_ref())
            .unwrap()
            .unwrap();
        let delayed_bytes = Bytes::from(delayed_bytes);

        let mut outputs = Vec::with_capacity(consumer_count + 1);
        let mut first_write_set = BTreeMap::new();
        first_write_set.insert(
            state_key.clone(),
            AbstractResourceWriteOp::WriteWithDelayedFields(WriteWithDelayedFieldsOp {
                write_op: WriteOp::Modification {
                    data: delayed_bytes.clone(),
                    metadata: StateValueMetadata::none(),
                },
                layout: layout.clone(),
                materialized_size: Some(delayed_bytes.len() as u64),
            }),
        );
        outputs.push((
            0,
            StarcoinTransactionOutput::new(
                VMOutput::new(
                    VMChangeSet::new(
                        first_write_set,
                        vec![],
                        BTreeMap::new(),
                        BTreeMap::new(),
                        BTreeMap::new(),
                    ),
                    ModuleWriteSet::empty(),
                    FeeStatement::zero(),
                    TransactionStatus::Keep(KeptVMStatus::Executed),
                    TransactionAuxiliaryData::None,
                ),
                HashMap::new(),
            ),
        ));

        for txn_idx in 1..=consumer_count {
            let mut write_set = BTreeMap::new();
            write_set.insert(
                state_key.clone(),
                AbstractResourceWriteOp::InPlaceDelayedFieldChange(InPlaceDelayedFieldChangeOp {
                    layout: layout.clone(),
                    materialized_size: delayed_bytes.len() as u64,
                    metadata: StateValueMetadata::none(),
                }),
            );
            outputs.push((
                txn_idx,
                StarcoinTransactionOutput::new(
                    VMOutput::new(
                        VMChangeSet::new(
                            write_set,
                            vec![],
                            BTreeMap::new(),
                            BTreeMap::new(),
                            BTreeMap::new(),
                        ),
                        ModuleWriteSet::empty(),
                        FeeStatement::zero(),
                        TransactionStatus::Keep(KeptVMStatus::Executed),
                        TransactionAuxiliaryData::None,
                    ),
                    HashMap::new(),
                ),
            ));
        }

        let delayed_fields = VersionedDelayedFields::empty();
        delayed_fields.set_base_value(delayed_id, DelayedFieldValue::Aggregator(7));

        (
            outputs,
            InMemoryStateView::new(HashMap::new()),
            delayed_fields,
            state_key,
        )
    }

    fn materialize_parallel_outputs_legacy_all_seq<S: StateView + Sync>(
        mut outputs: Vec<(usize, StarcoinTransactionOutput)>,
        delayed_fields: VersionedDelayedFields<DelayedFieldID>,
        delayed_field_cache: Arc<DelayedFieldCache>,
        state_view: &S,
        max_value_nest_depth: Option<u64>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        outputs.sort_by_key(|(idx, _)| *idx);
        let mut state_cache = StateViewCache::new(state_view);
        let mut group_cache: HashMap<StateKey, BTreeMap<StructTag, Bytes>> = HashMap::new();
        let mut results = Vec::with_capacity(outputs.len());

        for (txn_idx, output) in outputs.into_iter() {
            let (mut vm_output, group_read_layouts) = output.into_inner();
            let has_delayed = vm_output.contains_delayed_fields();
            let has_agg_v1 = !vm_output.aggregator_v1_delta_set().is_empty();
            let has_group_ops = vm_output.resource_write_set().values().any(|op| {
                matches!(
                    op,
                    AbstractResourceWriteOp::WriteResourceGroup(_)
                        | AbstractResourceWriteOp::ResourceGroupInPlaceDelayedFieldChange(_)
                )
            });

            let txn_output = if !has_delayed && !has_agg_v1 && !has_group_ops {
                vm_output.into_transaction_output().map_err(|err| {
                    VMStatus::error(
                        StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                        Some(err.to_string()),
                    )
                })?
            } else {
                if has_delayed || has_agg_v1 {
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
                    max_value_nest_depth,
                )?;
                let patched_events = if has_delayed {
                    materialize_events(&vm_output, &mapping, max_value_nest_depth)?
                } else {
                    vm_output
                        .events()
                        .iter()
                        .map(|(event, _)| event.clone())
                        .collect()
                };
                vm_output
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
                    })?
            };

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

    #[test]
    fn mixed_materialization_matches_legacy_all_seq_outputs() {
        let (new_outputs, new_state_view, agg_key) = build_sparse_conflict_case(80, 7, 71, 24, 128);
        let (legacy_outputs, legacy_state_view, legacy_agg_key) =
            build_sparse_conflict_case(80, 7, 71, 24, 128);
        assert_eq!(agg_key, legacy_agg_key);

        let mixed = materialize_parallel_outputs(
            new_outputs,
            VersionedDelayedFields::empty(),
            Arc::new(DelayedFieldCache::default()),
            &new_state_view,
            Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
        )
        .unwrap();
        let legacy = materialize_parallel_outputs_legacy_all_seq(
            legacy_outputs,
            VersionedDelayedFields::empty(),
            Arc::new(DelayedFieldCache::default()),
            &legacy_state_view,
            Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
        )
        .unwrap();
        assert_eq!(mixed, legacy);
    }

    #[test]
    fn delayed_only_dependency_materialization_matches_legacy_all_seq() {
        let (mixed_inputs, mixed_state_view, mixed_delayed_fields, mixed_key) =
            build_delayed_only_dependency_case(64);
        let (legacy_inputs, legacy_state_view, legacy_delayed_fields, legacy_key) =
            build_delayed_only_dependency_case(64);
        assert_eq!(mixed_key, legacy_key);

        let mixed = materialize_parallel_outputs(
            mixed_inputs,
            mixed_delayed_fields,
            Arc::new(DelayedFieldCache::default()),
            &mixed_state_view,
            Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
        )
        .expect("delayed-only dependency case should materialize successfully");
        let legacy = materialize_parallel_outputs_legacy_all_seq(
            legacy_inputs,
            legacy_delayed_fields,
            Arc::new(DelayedFieldCache::default()),
            &legacy_state_view,
            Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
        )
        .expect("legacy all-seq materialization should succeed");

        assert_eq!(mixed, legacy);
    }

    #[test]
    #[ignore = "manual benchmark; run with -- --ignored --nocapture"]
    fn bench_mixed_materialization_sparse_conflicts() {
        const TXN_COUNT: usize = 100;
        const CONFLICT_A: usize = 9;
        const CONFLICT_B: usize = 87;
        const MEMBERS_PER_GROUP: usize = 32;
        const BYTES_PER_MEMBER: usize = 256;
        const ROUNDS: usize = 8;

        // Warmup both paths once.
        {
            let (outputs, state_view, _) = build_sparse_conflict_case(
                TXN_COUNT,
                CONFLICT_A,
                CONFLICT_B,
                MEMBERS_PER_GROUP,
                BYTES_PER_MEMBER,
            );
            let _ = materialize_parallel_outputs_legacy_all_seq(
                outputs,
                VersionedDelayedFields::empty(),
                Arc::new(DelayedFieldCache::default()),
                &state_view,
                Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
            )
            .unwrap();
        }
        {
            let (outputs, state_view, _) = build_sparse_conflict_case(
                TXN_COUNT,
                CONFLICT_A,
                CONFLICT_B,
                MEMBERS_PER_GROUP,
                BYTES_PER_MEMBER,
            );
            let _ = materialize_parallel_outputs(
                outputs,
                VersionedDelayedFields::empty(),
                Arc::new(DelayedFieldCache::default()),
                &state_view,
                Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
            )
            .unwrap();
        }

        let mut legacy_total = Duration::ZERO;
        let mut mixed_total = Duration::ZERO;

        for _ in 0..ROUNDS {
            let (legacy_inputs, legacy_state_view, _) = build_sparse_conflict_case(
                TXN_COUNT,
                CONFLICT_A,
                CONFLICT_B,
                MEMBERS_PER_GROUP,
                BYTES_PER_MEMBER,
            );
            let legacy_start = Instant::now();
            let legacy_outputs = materialize_parallel_outputs_legacy_all_seq(
                legacy_inputs,
                VersionedDelayedFields::empty(),
                Arc::new(DelayedFieldCache::default()),
                &legacy_state_view,
                Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
            )
            .unwrap();
            legacy_total += legacy_start.elapsed();

            let (mixed_inputs, mixed_state_view, _) = build_sparse_conflict_case(
                TXN_COUNT,
                CONFLICT_A,
                CONFLICT_B,
                MEMBERS_PER_GROUP,
                BYTES_PER_MEMBER,
            );
            let mixed_start = Instant::now();
            let mixed_outputs = materialize_parallel_outputs(
                mixed_inputs,
                VersionedDelayedFields::empty(),
                Arc::new(DelayedFieldCache::default()),
                &mixed_state_view,
                Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
            )
            .unwrap();
            mixed_total += mixed_start.elapsed();

            assert_eq!(legacy_outputs, mixed_outputs);
        }

        let legacy_avg_ms = legacy_total.as_secs_f64() * 1000.0 / ROUNDS as f64;
        let mixed_avg_ms = mixed_total.as_secs_f64() * 1000.0 / ROUNDS as f64;
        let speedup = legacy_avg_ms / mixed_avg_ms;
        eprintln!(
            "materialize benchmark sparse conflicts: txns={} conflicts=2 rounds={} legacy_avg_ms={:.3} mixed_avg_ms={:.3} speedup={:.3}x",
            TXN_COUNT, ROUNDS, legacy_avg_ms, mixed_avg_ms, speedup
        );
    }
}
