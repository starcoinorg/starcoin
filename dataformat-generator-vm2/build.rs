//! Starcoin VM2 Data Format Generator
//!
//! This crate generates BCS schema for VM2 types.
//! TypeTag and related types are manually defined due to
//! serde-reflection issues with recursive enum types.
//!
//! Generated files:
//! - etc/starcoin_vm2_types.yml  - VM2 types schema

use serde_reflection::{Error, Samples, Tracer, TracerConfig};
use starcoin_crypto::HashValue;
use starcoin_vm2_vm_types::{
    account_address::AccountAddress,
    event::EventKey,
    language_storage::ModuleId,
    on_chain_resource::ChainId,
    transaction::{Module, TransactionArgument},
};

fn main() {
    match generate_vm2_types() {
        Ok(_) => (),
        Err(e) => println!("cargo:warning=VM2 types generation failed: {:?}", e),
    }
}

fn generate_vm2_types() -> Result<(), Error> {
    let config = TracerConfig::default().is_human_readable(false);
    let mut tracer = Tracer::new(config);
    let mut samples = Samples::new();

    tracer.trace_value(&mut samples, &HashValue::zero())?;
    tracer.trace_value(&mut samples, &AccountAddress::ZERO)?;

    tracer.trace_type::<ChainId>(&samples)?;
    // Note: WriteOp is complex in VM2, skipping auto-trace

    tracer.trace_value(&mut samples, &EventKey::new(0, AccountAddress::ZERO))?;

    tracer.trace_type::<ModuleId>(&samples)?;

    tracer.trace_type::<TransactionArgument>(&samples)?;
    tracer.trace_type::<Module>(&samples)?;
    // Note: EntryFunction, TransactionPayload depend on TypeTag/StructTag
    // They will be manually defined in the YAML file

    let registry = tracer.registry()?;
    let data = serde_yaml::to_string(&registry).unwrap();

    // Manually append TypeTag, StructTag, EntryFunction, TransactionPayload, WriteOp definitions
    // These cannot be auto-traced due to serde-reflection issues with recursive enum types
    let manual_defs = r#"
WriteOp:
  ENUM:
    0:
      Deletion: UNIT
    1:
      Value:
        NEWTYPE: BYTES
TypeTag:
  ENUM:
    0:
      Bool: UNIT
    1:
      U8: UNIT
    2:
      U64: UNIT
    3:
      U128: UNIT
    4:
      Address: UNIT
    5:
      Signer: UNIT
    6:
      Vector:
        NEWTYPE:
          TYPENAME: TypeTag
    7:
      Struct:
        NEWTYPE:
          TYPENAME: StructTag
    8:
      U16: UNIT
    9:
      U32: UNIT
    10:
      U256: UNIT
StructTag:
  STRUCT:
    - address:
        TYPENAME: AccountAddress
    - module:
        TYPENAME: Identifier
    - name:
        TYPENAME: Identifier
    - type_params:
        SEQ:
          TYPENAME: TypeTag
EntryFunction:
  STRUCT:
    - module:
        TYPENAME: ModuleId
    - function:
        TYPENAME: Identifier
    - ty_args:
        SEQ:
          TYPENAME: TypeTag
    - args:
        SEQ: BYTES
TransactionPayload:
  ENUM:
    0:
      EntryFunction:
        NEWTYPE:
          TYPENAME: EntryFunction
"#;

    let full_data = format!("{}{}", data, manual_defs);
    std::fs::write("../etc/starcoin_vm2_types.yml", full_data).unwrap();

    Ok(())
}
