use anyhow::Result;
use serde_reflection::{Error, Samples, Tracer, TracerConfig};
use starcoin_crypto::ed25519::Ed25519PrivateKey;
use starcoin_crypto::multi_ed25519::MultiEd25519PrivateKey;
use starcoin_crypto::{HashValue, PrivateKey, SigningKey, Uniform};
use starcoin_rpc_api::types::pubsub::Kind;
use starcoin_types::access_path::{AccessPath, DataType};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::AccountResource;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::contract_event::{ContractEvent, ContractEventV0};
use starcoin_types::event::EventKey;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::transaction::authenticator::TransactionAuthenticator;
use starcoin_types::transaction::{
    Module, Package, Script, ScriptABI, SignedUserTransaction, Transaction, TransactionArgument,
    TransactionPayload,
};
use starcoin_types::write_set::{WriteOp, WriteSet};

fn main() {
    generate().unwrap();
}

fn generate() -> Result<(), Error> {
    let mut tracer = Tracer::new(TracerConfig::default());
    let mut samples = Samples::new();
    tracer.trace_type::<AccessPath>(&samples)?;
    tracer.trace_value(&mut samples, &HashValue::zero())?;
    {
        let pri_key = Ed25519PrivateKey::generate_for_testing();
        tracer.trace_value(&mut samples, &pri_key)?;
        tracer.trace_value(&mut samples, &pri_key.public_key())?;
        tracer.trace_value(&mut samples, &pri_key.sign(&AccountAddress::random()))?;
    }
    {
        let pri_key = MultiEd25519PrivateKey::generate_for_testing();
        tracer.trace_value(&mut samples, &pri_key)?;
        tracer.trace_value(&mut samples, &pri_key.public_key())?;
        tracer.trace_value(&mut samples, &pri_key.sign(&AccountAddress::random()))?;
    }

    tracer.trace_type::<BlockMetadata>(&samples)?;

    tracer.trace_value(&mut samples, &EventKey::random())?;
    tracer.trace_type::<ContractEventV0>(&samples)?;
    tracer.trace_type::<ContractEvent>(&samples)?;
    tracer.trace_type::<WriteSet>(&samples)?;

    tracer.trace_type::<TransactionArgument>(&samples)?;
    tracer.trace_type::<TransactionAuthenticator>(&samples)?;
    tracer.trace_type::<TransactionPayload>(&samples)?;
    tracer.trace_type::<TypeTag>(&samples)?;
    tracer.trace_type::<WriteOp>(&samples)?;
    tracer.trace_type::<Script>(&samples)?;
    tracer.trace_type::<Module>(&samples)?;
    tracer.trace_type::<Package>(&samples)?;
    tracer.trace_type::<SignedUserTransaction>(&samples)?;
    tracer.trace_type::<Transaction>(&samples)?;

    tracer.trace_type::<AccountResource>(&samples)?;

    {
        tracer.trace_type::<Kind>(&samples)?;
    }

    tracer.trace_type::<AccessPath>(&samples)?;
    tracer.trace_type::<DataType>(&samples)?;
    tracer.trace_type::<ScriptABI>(&samples)?;
    let registry = tracer.registry()?;
    let data = serde_yaml::to_string(&registry).unwrap();
    std::fs::write("../etc/starcoin_types.yml", &data).unwrap();
    // println!("{}", data);
    Ok(())
}
