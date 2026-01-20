//! Starcoin VM1 Data Format Generator
//!
//! This crate is EXCLUDED from the main workspace to isolate VM1 dependencies.
//! This allows TypeTag and other types to be properly traced without VM2 conflicts.
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
    language_storage::{ModuleId, StructTag, TypeTag},
    transaction::{Module, Script, ScriptFunction, TransactionArgument, TransactionPayload},
    write_set::WriteOp,
};

fn main() {
    match generate_vm1_types() {
        Ok(_) => (),
        Err(e) => println!("cargo:warning=VM1 types generation failed: {:?}", e),
    }

    match generate_onchain_events() {
        Ok(_) => (),
        Err(e) => println!("cargo:warning=onchain_events generation failed: {:?}", e),
    }
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

    // Auto-trace TypeTag and related types (works because VM1 types are isolated)
    tracer.trace_type::<TypeTag>(&samples)?;
    tracer.trace_type::<StructTag>(&samples)?;

    tracer.trace_type::<TransactionArgument>(&samples)?;
    tracer.trace_type::<Module>(&samples)?;
    tracer.trace_type::<Script>(&samples)?;
    tracer.trace_type::<ScriptFunction>(&samples)?;
    tracer.trace_type::<TransactionPayload>(&samples)?;

    tracer.trace_type::<BalanceResource>(&samples)?;

    let registry = tracer.registry()?;
    let data = serde_yaml::to_string(&registry).unwrap();
    std::fs::write("../etc/starcoin_types.yml", data).unwrap();

    Ok(())
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
