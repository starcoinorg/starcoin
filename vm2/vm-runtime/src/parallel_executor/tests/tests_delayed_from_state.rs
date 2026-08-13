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
use starcoin_aggregator::types::DelayedFieldValue;
use starcoin_vm_runtime_types::change_set::VMChangeSet;
use starcoin_vm_runtime_types::module_write_set::ModuleWriteSet;
use starcoin_vm_types::fee_statement::FeeStatement;
use starcoin_vm_types::state_store::in_memory_state_view::InMemoryStateView;
use starcoin_vm_types::state_store::state_value::StateValue;
use starcoin_vm_types::state_store::state_value::StateValueMetadata;
use starcoin_vm_types::transaction::TransactionAuxiliaryData;

fn build_delayed_only_from_state_case(
    txn_count: usize,
) -> (
    Vec<(usize, StarcoinTransactionOutput)>,
    InMemoryStateView,
    VersionedDelayedFields<DelayedFieldID>,
) {
    let address = AccountAddress::from_hex_literal("0x1").unwrap();
    let struct_tag = StructTag {
        address,
        module: Identifier::new("DelayedOnly").unwrap(),
        name: Identifier::new("FromState").unwrap(),
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
    let delayed_id = DelayedFieldID::new_with_width(202, 8);
    let state_value = Value::struct_(Struct::pack(vec![
        Value::u64(5),
        Value::delayed_value(delayed_id),
    ]));
    let state_bytes = ValueSerDeContext::new(Some(DEFAULT_MAX_VALUE_NEST_DEPTH))
        .with_delayed_fields_serde()
        .serialize(&state_value, layout.as_ref())
        .unwrap()
        .unwrap();
    let state_bytes = Bytes::from(state_bytes);

    let mut outputs = Vec::with_capacity(txn_count);
    for txn_idx in 0..txn_count {
        let mut write_set = BTreeMap::new();
        write_set.insert(
            state_key.clone(),
            AbstractResourceWriteOp::InPlaceDelayedFieldChange(InPlaceDelayedFieldChangeOp {
                layout: layout.clone(),
                materialized_size: state_bytes.len() as u64,
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
    delayed_fields.set_base_value(delayed_id, DelayedFieldValue::Aggregator(11));

    let mut state_data = HashMap::new();
    state_data.insert(state_key, StateValue::new_legacy(state_bytes));

    (outputs, InMemoryStateView::new(state_data), delayed_fields)
}

#[test]
fn delayed_only_from_state_without_cached_base_should_materialize() {
    let (outputs, state_view, delayed_fields) = build_delayed_only_from_state_case(32);
    let _ = materialize_parallel_outputs(
        outputs,
        delayed_fields,
        Arc::new(DelayedFieldCache::default()),
        &state_view,
        Some(DEFAULT_MAX_VALUE_NEST_DEPTH),
    )
    .expect("delayed-only in-place changes should materialize from state base");
}
