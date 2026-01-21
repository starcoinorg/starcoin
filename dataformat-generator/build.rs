//! Starcoin VM1 Data Format Generator
//!
//! Generated files:
//! - etc/starcoin_types.yml      - VM1 types schema
//! - etc/onchain_events.yml      - On-chain event types

use move_core_types::u256::U256;
use serde_reflection::{
    ContainerFormat, Error, Format, Named, Samples, Tracer, TracerConfig, VariantFormat,
};
use starcoin_crypto::HashValue;
use starcoin_vm_types::{
    account_address::AccountAddress,
    account_config::BalanceResource,
    event::EventKey,
    genesis_config::ChainId,
    language_storage::{ModuleId, StructTag, TypeTag},
    transaction::{Module, TransactionArgument},
    write_set::WriteOp,
};

fn main() {
    generate_vm1_types().expect("VM1 types generation failed");
    generate_onchain_events().expect("onchain_events generation failed");
}

fn generate_vm1_types() -> Result<(), Error> {
    let config = TracerConfig::default().is_human_readable(false);
    let mut tracer = Tracer::new(config);
    let mut samples = Samples::new();

    tracer.trace_value(&mut samples, &HashValue::zero())?;
    tracer.trace_value(&mut samples, &AccountAddress::ZERO)?;

    tracer.trace_type::<ChainId>(&samples)?;
    tracer.trace_type::<WriteOp>(&samples)?;

    tracer.trace_value(&mut samples, &EventKey::new([0u8; 24]))?;

    tracer.trace_type::<ModuleId>(&samples)?;
    trace_transaction_arguments(&mut tracer, &mut samples)?;

    // Trace non-recursive types
    tracer.trace_type::<Module>(&samples)?;
    tracer.trace_type::<BalanceResource>(&samples)?;

    // Build registry and patch in recursive types manually.
    let mut registry = tracer.registry()?;
    registry.insert("TypeTag".to_string(), type_tag_format());
    registry.insert("StructTag".to_string(), struct_tag_format());
    registry.insert("Script".to_string(), script_format());
    registry.insert("ScriptFunction".to_string(), script_function_format());
    registry.insert("Package".to_string(), package_format());
    registry.insert(
        "TransactionPayload".to_string(),
        transaction_payload_format(),
    );

    assert_type_tag_layout();

    let data = serde_yaml::to_string(&registry).unwrap();

    std::fs::write("../etc/starcoin_types.yml", data).unwrap();

    Ok(())
}

fn trace_transaction_arguments(tracer: &mut Tracer, samples: &mut Samples) -> Result<(), Error> {
    let variants = [
        TransactionArgument::U8(1),
        TransactionArgument::U64(2),
        TransactionArgument::U128(3),
        TransactionArgument::Address(AccountAddress::ZERO),
        TransactionArgument::U8Vector(vec![0x1, 0x2]),
        TransactionArgument::Bool(true),
        TransactionArgument::U16(4),
        TransactionArgument::U32(5),
        TransactionArgument::U256(U256::one()),
    ];

    for arg in variants {
        tracer.trace_value(samples, &arg)?;
    }

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
            name: "type_params".into(),
            value: Format::Seq(Box::new(Format::TypeName("TypeTag".into()))),
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

fn script_function_format() -> ContainerFormat {
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
            value: Format::Option(Box::new(Format::TypeName("ScriptFunction".into()))),
        },
    ])
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
            name: "ScriptFunction".into(),
            value: VariantFormat::NewType(Box::new(Format::TypeName("ScriptFunction".into()))),
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
                type_params: vec![],
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

fn generate_onchain_events() -> Result<(), Error> {
    use starcoin_vm_types::account_config::{
        accept_token_payment::AcceptTokenEvent, block::NewBlockEvent, BlockRewardEvent, BurnEvent,
        DepositEvent, MintEvent, ProposalCreatedEvent, VoteChangedEvent, WithdrawEvent,
    };

    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();

    tracer.trace_type::<AcceptTokenEvent>(&samples)?;
    tracer.trace_type::<BlockRewardEvent>(&samples)?;
    tracer.trace_type::<BurnEvent>(&samples)?;
    tracer.trace_type::<DepositEvent>(&samples)?;
    tracer.trace_type::<MintEvent>(&samples)?;
    tracer.trace_type::<NewBlockEvent>(&samples)?;
    tracer.trace_type::<ProposalCreatedEvent>(&samples)?;
    tracer.trace_type::<VoteChangedEvent>(&samples)?;
    tracer.trace_type::<WithdrawEvent>(&samples)?;

    let registry = tracer.registry()?;
    let data = serde_yaml::to_string(&registry).unwrap();
    std::fs::write("../etc/onchain_events.yml", data).unwrap();

    Ok(())
}
