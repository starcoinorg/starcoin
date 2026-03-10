use move_binary_format::errors::PartialVMError;
use move_core_types::value::{IdentifierMappingKind, MoveStructLayout, MoveTypeLayout};
use move_vm_types::delayed_values::delayed_field_id::{DelayedFieldID, TryFromMoveValue};
use move_vm_types::value_serde::{ValueSerDeContext, ValueToIdentifierMapping};
use starcoin_aggregator::types::DelayedFieldValue;
use std::cell::RefCell;

fn nested_layout(kind: IdentifierMappingKind) -> MoveTypeLayout {
    MoveTypeLayout::Struct(MoveStructLayout::Runtime(vec![MoveTypeLayout::Struct(
        MoveStructLayout::Runtime(vec![
            MoveTypeLayout::Native(kind, Box::new(MoveTypeLayout::U64)),
            MoveTypeLayout::U64,
        ]),
    )]))
}

struct Mapping {
    id: DelayedFieldID,
    seen: RefCell<Option<DelayedFieldValue>>,
}

impl ValueToIdentifierMapping for Mapping {
    fn value_to_identifier(
        &self,
        kind: &IdentifierMappingKind,
        layout: &MoveTypeLayout,
        value: move_vm_types::values::Value,
    ) -> Result<DelayedFieldID, PartialVMError> {
        let (base, width) = DelayedFieldValue::try_from_move_value(layout, value, kind)?;
        assert_eq!(width, 8, "baseline expects u64 delayed field width");
        *self.seen.borrow_mut() = Some(base);
        Ok(self.id)
    }

    fn identifier_to_value(
        &self,
        _layout: &MoveTypeLayout,
        _identifier: DelayedFieldID,
    ) -> Result<move_vm_types::values::Value, PartialVMError> {
        unreachable!("baseline tests only exercise value->identifier exchange")
    }
}

#[test]
fn baseline_nested_aggregator_exchange_keeps_tail_bytes() {
    let layout = nested_layout(IdentifierMappingKind::Aggregator);
    let delayed_id = DelayedFieldID::new_with_width(42, 8);
    let mapping = Mapping {
        id: delayed_id,
        seen: RefCell::new(None),
    };

    let mut input = Vec::new();
    input.extend_from_slice(&123u64.to_le_bytes());
    input.extend_from_slice(&456u64.to_le_bytes());

    let value = ValueSerDeContext::new(None)
        .with_delayed_fields_replacement(&mapping)
        .deserialize(&input, &layout)
        .expect("exchange should succeed for nested aggregator layout");
    let output = ValueSerDeContext::new(None)
        .with_delayed_fields_serde()
        .serialize(&value, &layout)
        .expect("serialization should succeed")
        .expect("serializer should produce bytes");

    assert_eq!(&output[0..8], &delayed_id.as_u64().to_le_bytes());
    assert_eq!(&output[8..16], &456u64.to_le_bytes());
    assert!(matches!(
        mapping.seen.borrow().as_ref(),
        Some(DelayedFieldValue::Aggregator(v)) if *v == 123
    ));
}

#[test]
fn baseline_nested_snapshot_exchange_keeps_tail_bytes() {
    let layout = nested_layout(IdentifierMappingKind::Snapshot);
    let delayed_id = DelayedFieldID::new_with_width(7, 8);
    let mapping = Mapping {
        id: delayed_id,
        seen: RefCell::new(None),
    };

    let mut input = Vec::new();
    input.extend_from_slice(&11u64.to_le_bytes());
    input.extend_from_slice(&22u64.to_le_bytes());

    let value = ValueSerDeContext::new(None)
        .with_delayed_fields_replacement(&mapping)
        .deserialize(&input, &layout)
        .expect("exchange should succeed for nested snapshot layout");
    let output = ValueSerDeContext::new(None)
        .with_delayed_fields_serde()
        .serialize(&value, &layout)
        .expect("serialization should succeed")
        .expect("serializer should produce bytes");

    assert_eq!(&output[0..8], &delayed_id.as_u64().to_le_bytes());
    assert_eq!(&output[8..16], &22u64.to_le_bytes());
    assert!(matches!(
        mapping.seen.borrow().as_ref(),
        Some(DelayedFieldValue::Snapshot(v)) if *v == 11
    ));
}

#[test]
fn baseline_nested_layout_rejects_invalid_bytes_length() {
    let layout = nested_layout(IdentifierMappingKind::Aggregator);
    let delayed_id = DelayedFieldID::new_with_width(9, 8);
    let mapping = Mapping {
        id: delayed_id,
        seen: RefCell::new(None),
    };

    let invalid = vec![0u8; 8];
    let value = ValueSerDeContext::new(None)
        .with_delayed_fields_replacement(&mapping)
        .deserialize(&invalid, &layout);
    assert!(
        value.is_none(),
        "invalid payload length should fail deserialization"
    );
}
