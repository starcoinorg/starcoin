//! Starcoin VM1 Data Format Generator
//!
//! This build script generates VM1-only YAML schema files for cross-language SDK generation.
//! It uses starcoin-vm-types directly (pure VM1 types) to avoid TypeTag conflicts with VM2.
//!
//! Generated files:
//! - etc/starcoin_types.yml      - VM1 types schema
//! - etc/onchain_events.yml      - On-chain event types

use serde_reflection::{Error, Samples, Tracer, TracerConfig};
use starcoin_crypto::HashValue;
use starcoin_vm_types::{
    account_address::AccountAddress,
    account_config::BalanceResource,
    event::EventKey,
    genesis_config::ChainId,
    language_storage::ModuleId,
    transaction::{Module, TransactionArgument},
    write_set::WriteOp,
};

fn main() {
    println!("cargo:warning=Starting VM1 data format generation...");

    // Generate VM1 types schema
    match generate_vm1_types() {
        Ok(_) => println!("cargo:warning=starcoin_types.yml generated successfully"),
        Err(e) => println!("cargo:warning=VM1 types generation failed: {:?}", e),
    }

    // Generate on-chain events schema
    match generate_onchain_events() {
        Ok(_) => println!("cargo:warning=onchain_events.yml generated successfully"),
        Err(e) => println!("cargo:warning=onchain_events generation failed: {:?}", e),
    }
}

/// Generate VM1 types schema
fn generate_vm1_types() -> Result<(), Error> {
    // Use is_human_readable = false for binary (BCS) serialization format
    // This matches how starcoin serializes data on-chain
    let config = TracerConfig::default().is_human_readable(false);
    let mut tracer = Tracer::new(config);
    let mut samples = Samples::new();

    // Basic types
    tracer.trace_value(&mut samples, &HashValue::zero())?;
    tracer.trace_value(&mut samples, &AccountAddress::ZERO)?;

    // Chain types
    tracer.trace_type::<ChainId>(&samples)?;
    tracer.trace_type::<WriteOp>(&samples)?;

    // Event types
    tracer.trace_value(&mut samples, &EventKey::new([0u8; 24]))?;

    // Language types
    tracer.trace_type::<ModuleId>(&samples)?;

    // Transaction types - only those that don't recursively include TypeTag
    tracer.trace_type::<TransactionArgument>(&samples)?;
    tracer.trace_type::<Module>(&samples)?;

    // Account types
    tracer.trace_type::<BalanceResource>(&samples)?;

    let registry = tracer.registry()?;
    let data = serde_yaml::to_string(&registry).unwrap();
    std::fs::write("../etc/starcoin_types.yml", data).unwrap();

    Ok(())
}

/// Generate on-chain events schema
fn generate_onchain_events() -> Result<(), Error> {
    use starcoin_vm_types::account_config::{
        accept_token_payment::AcceptTokenEvent, BlockRewardEvent, BurnEvent, DepositEvent,
        MintEvent, ProposalCreatedEvent, VoteChangedEvent, WithdrawEvent,
    };

    let mut tracer = Tracer::new(TracerConfig::default());
    let samples = Samples::new();

    // On-chain event types
    tracer.trace_type::<AcceptTokenEvent>(&samples)?;
    tracer.trace_type::<BlockRewardEvent>(&samples)?;
    tracer.trace_type::<BurnEvent>(&samples)?;
    tracer.trace_type::<DepositEvent>(&samples)?;
    tracer.trace_type::<MintEvent>(&samples)?;
    tracer.trace_type::<ProposalCreatedEvent>(&samples)?;
    tracer.trace_type::<VoteChangedEvent>(&samples)?;
    tracer.trace_type::<WithdrawEvent>(&samples)?;

    let registry = tracer.registry()?;
    let data = serde_yaml::to_string(&registry).unwrap();
    std::fs::write("../etc/onchain_events.yml", data).unwrap();

    Ok(())
}
