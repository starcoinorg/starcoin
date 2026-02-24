use move_core_types::value::{IdentifierMappingKind, MoveStructLayout, MoveTypeLayout};
use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
use starcoin_aggregator::types::DelayedFieldValue;
use starcoin_vm_runtime::data_cache::{
    manual_exchange_bytes_for_nested_native_u64, nested_native_u64_kind_for_manual_exchange,
};

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
