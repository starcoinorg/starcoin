use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_executor::execute_readonly_function;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::genesis_address;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_vm_types::value::MoveValue;
use test_helper::executor::prepare_genesis;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DataProof {
    address: AccountAddress,
    index: u64,
    proof: Vec<String>,
}

#[stest::test]
fn test_genesis_nft_verify() -> Result<()> {
    assert!(verify_genesis_nft_address(genesis_address())?);
    assert!(verify_genesis_nft_address(
        AccountAddress::from_hex_literal("0x86fddffbbb603c428e5c74442ce1e966")?
    )?);
    Ok(())
}

fn verify_genesis_nft_address(mint_address: AccountAddress) -> Result<bool> {
    let (chain_state, _net) = prepare_genesis();
    let merkle_data = include_str!("genesis-nft-address.json");
    let merkle_data: serde_json::Value = serde_json::from_str(merkle_data)?;
    let proofs: Vec<DataProof> = serde_json::from_value(merkle_data["proofs"].clone())?;

    let mint_proof = proofs.iter().find(|p| p.address == mint_address).unwrap();
    let index = MoveValue::U64(mint_proof.index);
    let proofs = MoveValue::Vector(
        mint_proof
            .proof
            .iter()
            .map(|p| {
                hex::decode(p.as_str().strip_prefix("0x").unwrap_or_else(|| p.as_str())).unwrap()
            })
            .map(MoveValue::vector_u8)
            .collect(),
    );

    let ret = execute_readonly_function(
        &chain_state,
        &ModuleId::new(genesis_address(), Identifier::new("GenesisNFT").unwrap()),
        &Identifier::new("verify").unwrap(),
        vec![],
        vec![
            MoveValue::Address(mint_address).simple_serialize().unwrap(),
            index.simple_serialize().unwrap(),
            proofs.simple_serialize().unwrap(),
        ],
        None,
    )?;
    let verified: bool = bcs_ext::from_bytes(ret[0].as_slice()).unwrap();
    Ok(verified)
}
