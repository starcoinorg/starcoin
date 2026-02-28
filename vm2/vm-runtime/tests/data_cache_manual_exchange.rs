use move_core_types::value::{IdentifierMappingKind, MoveStructLayout, MoveTypeLayout};
use move_vm_types::delayed_values::delayed_field_id::{DelayedFieldID, TryFromMoveValue};
use move_vm_types::value_serde::{
    deserialize_and_replace_values_with_ids, serialize_and_allow_delayed_values,
    ValueToIdentifierMapping,
};
use starcoin_aggregator::types::DelayedFieldValue;
use starcoin_vm_runtime::data_cache::{
    manual_exchange_bytes_for_nested_native_u64, nested_native_u64_kind_for_manual_exchange,
};
use std::cell::RefCell;

fn nested_manual_exchange_layout(kind: IdentifierMappingKind) -> MoveTypeLayout {
    MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![MoveTypeLayout::Struct(
        MoveStructLayout::Runtime(vec![
            MoveTypeLayout::Native(kind, Box::new(MoveTypeLayout::U64)),
            MoveTypeLayout::U64,
        ]),
    )]))
}

#[test]
fn nested_native_u64_layout_detection_matches_supported_kinds() {
    let agg = nested_manual_exchange_layout(IdentifierMappingKind::Aggregator);
    let snapshot = nested_manual_exchange_layout(IdentifierMappingKind::Snapshot);

    assert!(matches!(
        nested_native_u64_kind_for_manual_exchange(&agg),
        Some(IdentifierMappingKind::Aggregator)
    ));
    assert!(matches!(
        nested_native_u64_kind_for_manual_exchange(&snapshot),
        Some(IdentifierMappingKind::Snapshot)
    ));
}

#[test]
fn nested_native_u64_layout_detection_rejects_non_target_layout() {
    let wrong_inner = MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![
        MoveTypeLayout::Native(
            IdentifierMappingKind::Aggregator,
            Box::new(MoveTypeLayout::U64),
        ),
        MoveTypeLayout::U128,
    ]));
    let layout = MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![wrong_inner]));

    assert!(nested_native_u64_kind_for_manual_exchange(&layout).is_none());
}

#[test]
fn manual_exchange_bytes_replaces_first_field_and_keeps_tail() {
    let delayed_id = DelayedFieldID::new_with_width(42, 8);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&7u64.to_le_bytes());
    bytes.extend_from_slice(&99u64.to_le_bytes());

    let (exchanged, delayed_value) = manual_exchange_bytes_for_nested_native_u64(
        IdentifierMappingKind::Aggregator,
        &bytes,
        delayed_id,
    )
    .expect("manual exchange should succeed");

    assert_eq!(&exchanged[0..8], &delayed_id.as_u64().to_le_bytes());
    assert_eq!(&exchanged[8..16], &99u64.to_le_bytes());
    assert!(matches!(
        delayed_value,
        DelayedFieldValue::Aggregator(v) if v == 7
    ));
}

#[test]
fn manual_exchange_bytes_rejects_invalid_length() {
    let delayed_id = DelayedFieldID::new_with_width(1, 8);
    let err = manual_exchange_bytes_for_nested_native_u64(
        IdentifierMappingKind::Snapshot,
        &[0u8; 8],
        delayed_id,
    )
    .expect_err("manual exchange should fail for invalid length");

    assert!(format!("{err:?}").contains("expected 16 bytes"));
}

#[test]
fn manual_exchange_bytes_rejects_invalid_delayed_field_width() {
    let delayed_id = DelayedFieldID::new_with_width(7, 16);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&1u64.to_le_bytes());
    bytes.extend_from_slice(&2u64.to_le_bytes());

    let err = manual_exchange_bytes_for_nested_native_u64(
        IdentifierMappingKind::Aggregator,
        &bytes,
        delayed_id,
    )
    .expect_err("manual exchange should reject non-u64 delayed field ids");

    assert!(format!("{err:?}").contains("expected delayed field width 8"));
}

#[test]
fn manual_exchange_bytes_snapshot_maps_to_snapshot_value() {
    let delayed_id = DelayedFieldID::new_with_width(7, 8);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&13u64.to_le_bytes());
    bytes.extend_from_slice(&21u64.to_le_bytes());

    let (_, delayed_value) = manual_exchange_bytes_for_nested_native_u64(
        IdentifierMappingKind::Snapshot,
        &bytes,
        delayed_id,
    )
    .expect("manual exchange should succeed");

    assert!(matches!(
        delayed_value,
        DelayedFieldValue::Snapshot(v) if v == 13
    ));
}

#[test]
fn manual_exchange_bytes_matches_standard_exchange_for_target_layout() {
    struct Mapping {
        id: DelayedFieldID,
        seen: RefCell<Option<DelayedFieldValue>>,
    }

    impl ValueToIdentifierMapping for Mapping {
        type Identifier = DelayedFieldID;

        fn value_to_identifier(
            &self,
            kind: &move_core_types::value::IdentifierMappingKind,
            layout: &MoveTypeLayout,
            value: move_vm_types::values::Value,
        ) -> Result<Self::Identifier, move_binary_format::errors::PartialVMError> {
            let (base, width) = DelayedFieldValue::try_from_move_value(layout, value, kind)?;
            assert_eq!(width, 8);
            *self.seen.borrow_mut() = Some(base);
            Ok(self.id)
        }

        fn identifier_to_value(
            &self,
            _layout: &MoveTypeLayout,
            _identifier: Self::Identifier,
        ) -> Result<move_vm_types::values::Value, move_binary_format::errors::PartialVMError>
        {
            unreachable!()
        }
    }

    let delayed_id = DelayedFieldID::new_with_width(33, 8);
    let layout = nested_manual_exchange_layout(IdentifierMappingKind::Aggregator);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&123u64.to_le_bytes());
    bytes.extend_from_slice(&456u64.to_le_bytes());

    let mapping = Mapping {
        id: delayed_id,
        seen: RefCell::new(None),
    };
    let standard = deserialize_and_replace_values_with_ids(&bytes, &layout, &mapping)
        .expect("standard exchange should succeed");
    let standard_bytes = serialize_and_allow_delayed_values(&standard, &layout)
        .expect("standard serialization should succeed")
        .expect("standard serialization should return bytes");
    let standard_base = mapping
        .seen
        .borrow()
        .clone()
        .expect("standard exchange should capture base value");

    let (manual_bytes, manual_base) = manual_exchange_bytes_for_nested_native_u64(
        IdentifierMappingKind::Aggregator,
        &bytes,
        delayed_id,
    )
    .expect("manual exchange should succeed");

    assert_eq!(manual_bytes, standard_bytes);
    assert_eq!(manual_base, standard_base);
}
