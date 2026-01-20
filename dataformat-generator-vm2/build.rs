//! Starcoin VM2 Data Format Generator
//!
//! This crate generates BCS schema for VM2 types.
//!
//! Generated files:
//! - etc/starcoin_vm2_types.yml  - VM2 types schema

use bytes::Bytes;
use serde_reflection::{
    ContainerFormat, Error, Format, Named, Samples, Tracer, TracerConfig, VariantFormat,
};
use starcoin_crypto::HashValue;
use starcoin_vm2_vm_types::{
    account_address::AccountAddress,
    event::EventKey,
    language_storage::{ModuleId, StructTag, TypeTag},
    on_chain_config::CurrentTimeMicroseconds,
    on_chain_resource::ChainId,
    state_store::state_value::PersistedStateValueMetadata,
    transaction::{Module, TransactionArgument},
    write_set::WriteOp,
};

fn main() {
    generate_vm2_types().expect("VM2 types generation failed");
}

fn generate_vm2_types() -> Result<(), Error> {
    let config = TracerConfig::default().is_human_readable(false);
    let mut tracer = Tracer::new(config);
    let mut samples = Samples::new();

    tracer.trace_value(&mut samples, &HashValue::zero())?;
    tracer.trace_value(&mut samples, &AccountAddress::ZERO)?;

    tracer.trace_type::<ChainId>(&samples)?;

    tracer.trace_value(&mut samples, &EventKey::new(0, AccountAddress::ZERO))?;

    tracer.trace_type::<ModuleId>(&samples)?;

    // Trace only non-recursive types
    tracer.trace_type::<TransactionArgument>(&samples)?;
    tracer.trace_type::<Module>(&samples)?;

    let mut registry = tracer.registry()?;
    registry.insert("TypeTag".to_string(), type_tag_format());
    registry.insert("StructTag".to_string(), struct_tag_format());
    registry.insert("Script".to_string(), script_format());
    registry.insert("EntryFunction".to_string(), entry_function_format());
    registry.insert("Package".to_string(), package_format());
    registry.insert("WriteOp".to_string(), write_op_format());
    registry.insert(
        "StateValueMetadata".to_string(),
        state_value_metadata_format(),
    );
    registry.insert(
        "TransactionPayload".to_string(),
        transaction_payload_format(),
    );

    assert_type_tag_layout();
    assert_state_value_metadata_layout();
    assert_write_op_layout();

    let data = serde_yaml::to_string(&registry).unwrap();

    std::fs::write("../etc/starcoin_vm2_types.yml", data).unwrap();

    Ok(())
}

fn type_tag_format() -> ContainerFormat {
    let mut variants = std::collections::BTreeMap::new();
    variants.insert(
        0,
        Named {
            name: "Bool".into(),
            value: VariantFormat::Unit,
        },
    );
    variants.insert(
        1,
        Named {
            name: "U8".into(),
            value: VariantFormat::Unit,
        },
    );
    variants.insert(
        2,
        Named {
            name: "U64".into(),
            value: VariantFormat::Unit,
        },
    );
    variants.insert(
        3,
        Named {
            name: "U128".into(),
            value: VariantFormat::Unit,
        },
    );
    variants.insert(
        4,
        Named {
            name: "Address".into(),
            value: VariantFormat::Unit,
        },
    );
    variants.insert(
        5,
        Named {
            name: "Signer".into(),
            value: VariantFormat::Unit,
        },
    );
    variants.insert(
        6,
        Named {
            name: "Vector".into(),
            value: VariantFormat::NewType(Box::new(Format::TypeName("TypeTag".into()))),
        },
    );
    variants.insert(
        7,
        Named {
            name: "Struct".into(),
            value: VariantFormat::NewType(Box::new(Format::TypeName("StructTag".into()))),
        },
    );
    variants.insert(
        8,
        Named {
            name: "U16".into(),
            value: VariantFormat::Unit,
        },
    );
    variants.insert(
        9,
        Named {
            name: "U32".into(),
            value: VariantFormat::Unit,
        },
    );
    variants.insert(
        10,
        Named {
            name: "U256".into(),
            value: VariantFormat::Unit,
        },
    );
    ContainerFormat::Enum(variants)
}

fn struct_tag_format() -> ContainerFormat {
    ContainerFormat::Struct(vec![
        Named {
            name: "address".into(),
            value: Format::TypeName("AccountAddress".into()),
        },
        Named {
            name: "module".into(),
            value: Format::TypeName("Identifier".into()),
        },
        Named {
            name: "name".into(),
            value: Format::TypeName("Identifier".into()),
        },
        Named {
            name: "type_args".into(),
            value: Format::Seq(Box::new(Format::TypeName("TypeTag".into()))),
        },
    ])
}

fn entry_function_format() -> ContainerFormat {
    ContainerFormat::Struct(vec![
        Named {
            name: "module".into(),
            value: Format::TypeName("ModuleId".into()),
        },
        Named {
            name: "function".into(),
            value: Format::TypeName("Identifier".into()),
        },
        Named {
            name: "ty_args".into(),
            value: Format::Seq(Box::new(Format::TypeName("TypeTag".into()))),
        },
        Named {
            name: "args".into(),
            value: Format::Seq(Box::new(Format::Bytes)),
        },
    ])
}

fn script_format() -> ContainerFormat {
    ContainerFormat::Struct(vec![
        Named {
            name: "code".into(),
            value: Format::Bytes,
        },
        Named {
            name: "ty_args".into(),
            value: Format::Seq(Box::new(Format::TypeName("TypeTag".into()))),
        },
        Named {
            name: "args".into(),
            value: Format::Seq(Box::new(Format::TypeName("TransactionArgument".into()))),
        },
    ])
}

fn package_format() -> ContainerFormat {
    ContainerFormat::Struct(vec![
        Named {
            name: "package_address".into(),
            value: Format::TypeName("AccountAddress".into()),
        },
        Named {
            name: "modules".into(),
            value: Format::Seq(Box::new(Format::TypeName("Module".into()))),
        },
        Named {
            name: "init_script".into(),
            value: Format::Option(Box::new(Format::TypeName("EntryFunction".into()))),
        },
    ])
}

fn write_op_format() -> ContainerFormat {
    use VariantFormat::*;

    let mut variants = std::collections::BTreeMap::new();
    variants.insert(
        0,
        Named {
            name: "Creation".into(),
            value: NewType(Box::new(Format::Bytes)),
        },
    );
    variants.insert(
        1,
        Named {
            name: "Modification".into(),
            value: NewType(Box::new(Format::Bytes)),
        },
    );
    variants.insert(
        2,
        Named {
            name: "Deletion".into(),
            value: Unit,
        },
    );
    variants.insert(
        3,
        Named {
            name: "CreationWithMetadata".into(),
            value: VariantFormat::Struct(vec![
                Named {
                    name: "data".into(),
                    value: Format::Bytes,
                },
                Named {
                    name: "metadata".into(),
                    value: Format::TypeName("StateValueMetadata".into()),
                },
            ]),
        },
    );
    variants.insert(
        4,
        Named {
            name: "ModificationWithMetadata".into(),
            value: VariantFormat::Struct(vec![
                Named {
                    name: "data".into(),
                    value: Format::Bytes,
                },
                Named {
                    name: "metadata".into(),
                    value: Format::TypeName("StateValueMetadata".into()),
                },
            ]),
        },
    );
    variants.insert(
        5,
        Named {
            name: "DeletionWithMetadata".into(),
            value: VariantFormat::Struct(vec![Named {
                name: "metadata".into(),
                value: Format::TypeName("StateValueMetadata".into()),
            }]),
        },
    );
    ContainerFormat::Enum(variants)
}

fn state_value_metadata_format() -> ContainerFormat {
    let mut variants = std::collections::BTreeMap::new();
    variants.insert(
        0,
        Named {
            name: "V0".into(),
            value: VariantFormat::Struct(vec![
                Named {
                    name: "deposit".into(),
                    value: Format::U64,
                },
                Named {
                    name: "creation_time_usecs".into(),
                    value: Format::U64,
                },
            ]),
        },
    );
    variants.insert(
        1,
        Named {
            name: "V1".into(),
            value: VariantFormat::Struct(vec![
                Named {
                    name: "slot_deposit".into(),
                    value: Format::U64,
                },
                Named {
                    name: "bytes_deposit".into(),
                    value: Format::U64,
                },
                Named {
                    name: "creation_time_usecs".into(),
                    value: Format::U64,
                },
            ]),
        },
    );
    ContainerFormat::Enum(variants)
}

fn transaction_payload_format() -> ContainerFormat {
    let mut variants = std::collections::BTreeMap::new();
    variants.insert(
        0,
        Named {
            name: "Script".into(),
            value: VariantFormat::NewType(Box::new(Format::TypeName("Script".into()))),
        },
    );
    variants.insert(
        1,
        Named {
            name: "Package".into(),
            value: VariantFormat::NewType(Box::new(Format::TypeName("Package".into()))),
        },
    );
    variants.insert(
        2,
        Named {
            name: "EntryFunction".into(),
            value: VariantFormat::NewType(Box::new(Format::TypeName("EntryFunction".into()))),
        },
    );

    ContainerFormat::Enum(variants)
}

fn assert_type_tag_layout() {
    use TypeTag::*;

    let cases = [
        (Bool, 0u8),
        (U8, 1),
        (U64, 2),
        (U128, 3),
        (Address, 4),
        (Signer, 5),
        (Vector(Box::new(Bool)), 6),
        (
            Struct(Box::new(StructTag {
                address: AccountAddress::ZERO,
                module: "Test".parse().unwrap(),
                name: "S".parse().unwrap(),
                type_args: vec![],
            })),
            7,
        ),
        (U16, 8),
        (U32, 9),
        (U256, 10),
    ];

    for (tag, expected) in cases {
        let bytes = bcs::to_bytes(&tag).expect("bcs serialize TypeTag");
        let disc = bytes.first().copied().unwrap_or(255);
        assert_eq!(
            disc, expected,
            "TypeTag variant index mismatch for {:?}",
            tag
        );
    }
}

fn assert_state_value_metadata_layout() {
    let v0 = PersistedStateValueMetadata::V0 {
        deposit: 0,
        creation_time_usecs: 0,
    };
    let v1 = PersistedStateValueMetadata::V1 {
        slot_deposit: 0,
        bytes_deposit: 0,
        creation_time_usecs: 0,
    };

    let b0 = bcs::to_bytes(&v0).expect("bcs serialize StateValueMetadata::V0");
    let b1 = bcs::to_bytes(&v1).expect("bcs serialize StateValueMetadata::V1");
    assert_eq!(
        b0.first().copied(),
        Some(0),
        "StateValueMetadata V0 discriminant mismatch"
    );
    assert_eq!(
        b1.first().copied(),
        Some(1),
        "StateValueMetadata V1 discriminant mismatch"
    );
}

fn assert_write_op_layout() {
    let creation = WriteOp::legacy_creation(Bytes::new());
    let modification = WriteOp::legacy_modification(Bytes::new());
    let deletion = WriteOp::legacy_deletion();

    let time = CurrentTimeMicroseconds { microseconds: 0 };
    let meta =
        starcoin_vm2_vm_types::state_store::state_value::StateValueMetadata::legacy(0, &time);

    let creation_meta = WriteOp::Creation {
        data: Bytes::new(),
        metadata: meta.clone(),
    };
    let modification_meta = WriteOp::Modification {
        data: Bytes::new(),
        metadata: meta.clone(),
    };
    let deletion_meta = WriteOp::Deletion { metadata: meta };

    let cases = [
        (creation, 0u8, "Creation"),
        (modification, 1, "Modification"),
        (deletion, 2, "Deletion"),
        (creation_meta, 3, "CreationWithMetadata"),
        (modification_meta, 4, "ModificationWithMetadata"),
        (deletion_meta, 5, "DeletionWithMetadata"),
    ];

    for (op, expected, label) in cases {
        let bytes = bcs::to_bytes(&op).expect("bcs serialize WriteOp");
        let disc = bytes.first().copied().unwrap_or(255);
        assert_eq!(
            disc, expected,
            "WriteOp variant index mismatch for {}",
            label
        );
    }
}
