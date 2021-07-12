use anyhow::Result;
use serde_reflection::{Error, Samples, Tracer, TracerConfig};
use starcoin_crypto::ed25519::Ed25519PrivateKey;
use starcoin_crypto::multi_ed25519::MultiEd25519PrivateKey;
use starcoin_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue, PrivateKey, SigningKey, Uniform,
};
// use starcoin_rpc_api::types::pubsub::Kind;
use starcoin_types::access_path::{AccessPath, DataPath, DataType};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::accept_token_payment::AcceptTokenEvent;
use starcoin_types::account_config::block::NewBlockEvent;
use starcoin_types::account_config::{
    AccountResource, BlockRewardEvent, BurnEvent, DepositEvent, MintEvent, ProposalCreatedEvent,
    VoteChangedEvent, WithdrawEvent,
};
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::contract_event::{ContractEvent, ContractEventV0};
use starcoin_types::event::EventKey;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::sign_message::{SignedMessage, SigningMessage};
use starcoin_types::transaction::authenticator::{AuthenticationKey, TransactionAuthenticator};
use starcoin_types::transaction::{
    Module, Package, Script, ScriptABI, SignedUserTransaction, Transaction, TransactionArgument,
    TransactionPayload,
};
use starcoin_types::write_set::{WriteOp, WriteSet};

fn main() {
    generate().unwrap();
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, CryptoHasher, CryptoHash)]
struct DummyObj(Vec<u8>);

impl Default for DummyObj {
    fn default() -> Self {
        DummyObj(vec![0; 32])
    }
}

fn generate() -> Result<(), Error> {
    let mut tracer = Tracer::new(TracerConfig::default());
    let mut samples = Samples::new();
    tracer.trace_type::<DataPath>(&samples)?;
    tracer.trace_type::<AccessPath>(&samples)?;
    tracer.trace_value(&mut samples, &HashValue::zero())?;
    {
        let pri_key = Ed25519PrivateKey::generate_for_testing();
        tracer.trace_value(&mut samples, &pri_key)?;
        tracer.trace_value(&mut samples, &pri_key.public_key())?;
        tracer.trace_value(&mut samples, &pri_key.sign(&DummyObj::default()))?;

        tracer.trace_value::<AuthenticationKey>(
            &mut samples,
            &AuthenticationKey::ed25519(&pri_key.public_key()),
        )?;
    }
    {
        let pri_key = MultiEd25519PrivateKey::generate_for_testing();
        tracer.trace_value(&mut samples, &pri_key)?;
        tracer.trace_value(&mut samples, &pri_key.public_key())?;
        tracer.trace_value(&mut samples, &pri_key.sign(&DummyObj::default()))?;
    }

    tracer.trace_type::<BlockMetadata>(&samples)?;

    tracer.trace_value(
        &mut samples,
        &EventKey::new_from_address(&AccountAddress::random(), 0),
    )?;
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

    // {
    //     tracer.trace_type::<Kind>(&samples)?;
    // }

    tracer.trace_type::<AccessPath>(&samples)?;
    tracer.trace_type::<DataType>(&samples)?;
    tracer.trace_type::<ScriptABI>(&samples)?;
    tracer.trace_type::<SigningMessage>(&samples)?;
    tracer.trace_type::<SignedMessage>(&samples)?;
    let registry = tracer.registry()?;
    let data = serde_yaml::to_string(&registry).unwrap();
    std::fs::write("../etc/starcoin_types.yml", &data).unwrap();

    {
        let mut tracer = Tracer::new(TracerConfig::default());
        let samples = Samples::new();
        tracer.trace_type::<WithdrawEvent>(&samples)?;
        tracer.trace_type::<DepositEvent>(&samples)?;
        tracer.trace_type::<AcceptTokenEvent>(&samples)?;
        tracer.trace_type::<BlockRewardEvent>(&samples)?;
        tracer.trace_type::<BurnEvent>(&samples)?;
        tracer.trace_type::<MintEvent>(&samples)?;
        tracer.trace_type::<ProposalCreatedEvent>(&samples)?;
        tracer.trace_type::<VoteChangedEvent>(&samples)?;
        tracer.trace_type::<NewBlockEvent>(&samples)?;
        let registry = tracer.registry()?;
        let data = serde_yaml::to_string(&registry).unwrap();
        std::fs::write("../etc/onchain_events.yml", &data).unwrap();
    }
    // println!("{}", data);
    Ok(())
}
