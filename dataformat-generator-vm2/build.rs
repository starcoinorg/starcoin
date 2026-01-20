//! Starcoin Data Format Generator
//!
//! This build script generates YAML schema files for cross-language SDK generation.
//!
//! Generated files:
//! - etc/starcoin_types.yml      - VM1 types schema (manual maintenance - cannot auto-generate due to VM1/VM2 conflicts)
//! - etc/starcoin_types_vm2.yml  - VM2 types schema (auto-generated)
//! - etc/onchain_events.yml      - On-chain event types (manual maintenance - depends on VM1)
//!
//! Note: VM1 schema generation is disabled because starcoin-vm1-types depends on VM2,
//! causing serde-reflection to fail with TypeTag enum variant conflicts.

use serde_reflection::{Error, Samples, Tracer, TracerConfig};

// VM2 types for starcoin_types_vm2.yml
use starcoin_vm2_vm_types::account_address::AccountAddress as AccountAddress2;
use starcoin_vm2_vm_types::block_metadata::BlockMetadata as BlockMetadata2;
use starcoin_vm2_vm_types::event::EventKey as EventKey2;
use starcoin_vm2_vm_types::transaction::{
    Module as Module2, TransactionArgument as TransactionArgument2,
};

// VM2 crypto
use starcoin_vm2_crypto::HashValue as HashValue2;

fn main() {
    // Generate VM2 types schema
    match generate_vm2_types() {
        Ok(_) => (),
        Err(e) => println!("cargo:warning=VM2 generation failed: {:?}", e),
    }
}

/// Generate VM2 types schema
fn generate_vm2_types() -> Result<(), Error> {
    let mut tracer = Tracer::new(TracerConfig::default());
    let mut samples = Samples::new();

    // Basic types
    tracer.trace_value(&mut samples, &HashValue2::zero())?;

    // VM2 core types - only basic types that don't involve TypeTag, StructTag, or crypto
    // Due to workspace having both VM1 and VM2 dependencies, many types cause issues

    tracer.trace_type::<BlockMetadata2>(&samples)?;
    tracer.trace_value(&mut samples, &EventKey2::new(0, AccountAddress2::random()))?;

    tracer.trace_type::<TransactionArgument2>(&samples)?;
    // WriteOp2 depends on StateValueMetadata which doesn't impl Deserialize - skip
    // tracer.trace_type::<WriteOp2>(&samples)?;
    tracer.trace_type::<Module2>(&samples)?;

    let registry = tracer.registry()?;
    let data = serde_yaml::to_string(&registry).unwrap();
    std::fs::write("../etc/starcoin_types_vm2.yml", data).unwrap();

    Ok(())
}
