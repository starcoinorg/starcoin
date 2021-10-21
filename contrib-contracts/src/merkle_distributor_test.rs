use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_executor::{execute_readonly_function, Account};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_vm_types::account_config::association_address;
use starcoin_vm_types::token::stc::stc_type_tag;
use starcoin_vm_types::transaction::{Package, ScriptFunction, TransactionPayload};
use starcoin_vm_types::value::MoveValue;
use test_helper::executor::{
    association_execute, association_execute_should_success, compile_modules_with_address,
    move_abort_code, prepare_genesis,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DataProof {
    address: AccountAddress,
    index: u64,
    amount: u128,
    /// proofs in hex string
    proof: Vec<String>,
}

#[stest::test]
fn test_merkle_distributor() -> Result<()> {
    let assocation = Account::new_association();
    println!("{}", assocation.address());
    let (chain_state, net) = prepare_genesis();
    let merkle_data = include_str!("merkle-test.json");
    let merkle_data: serde_json::Value = serde_json::from_str(merkle_data)?;
    let root = merkle_data["root"].as_str().unwrap();
    let root_in_bytes = hex::decode(root.strip_prefix("0x").unwrap_or(root))?;
    let proofs: Vec<DataProof> = serde_json::from_value(merkle_data["proofs"].clone())?;

    // deploy the module
    {
        let source = include_str!("../modules/MerkleDistributor.move");
        let modules = compile_modules_with_address(association_address(), source);

        let package = Package::new(modules, None)?;
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::Package(package),
        )?;
    }

    // association: create the merkle distributor.
    {
        let merkle_root = MoveValue::vector_u8(root_in_bytes);

        let rewards_total = MoveValue::U128(proofs.iter().map(|p| p.amount).sum());
        let leafs = MoveValue::U64(proofs.len() as u64);

        let script_function = ScriptFunction::new(
            ModuleId::new(
                association_address(),
                Identifier::new("MerkleDistributorScripts").unwrap(),
            ),
            Identifier::new("create").unwrap(),
            vec![stc_type_tag()],
            vec![
                merkle_root.simple_serialize().unwrap(),
                rewards_total.simple_serialize().unwrap(),
                leafs.simple_serialize().unwrap(),
            ],
        );

        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
    }

    // check I'm not claimed.
    {
        let distribution_address = MoveValue::Address(*assocation.address());
        let index = MoveValue::U64(0);

        let ret = execute_readonly_function(
            &chain_state,
            &ModuleId::new(
                association_address(),
                Identifier::new("MerkleDistributor").unwrap(),
            ),
            &Identifier::new("is_claimed").unwrap(),
            vec![stc_type_tag()],
            vec![
                distribution_address.simple_serialize().unwrap(),
                index.simple_serialize().unwrap(),
            ],
            None,
        )?;
        let is_claimed: bool = bcs_ext::from_bytes(ret[0].as_slice()).unwrap();
        assert!(!is_claimed, "should not claimed");
    }

    // claim more than what you get should error.
    {
        let association_proof = proofs
            .iter()
            .find(|p| p.address == *assocation.address())
            .unwrap();

        let distribution_address = MoveValue::Address(*assocation.address());
        let index = MoveValue::U64(association_proof.index);
        let account = MoveValue::Address(*assocation.address());
        let amount = MoveValue::U128(association_proof.amount + 1);
        let proofs = MoveValue::Vector(
            association_proof
                .proof
                .iter()
                .map(|p| {
                    hex::decode(p.as_str().strip_prefix("0x").unwrap_or_else(|| p.as_str()))
                        .unwrap()
                })
                .map(MoveValue::vector_u8)
                .collect(),
        );
        let script_function = ScriptFunction::new(
            ModuleId::new(
                association_address(),
                Identifier::new("MerkleDistributorScripts").unwrap(),
            ),
            Identifier::new("claim_for_address").unwrap(),
            vec![stc_type_tag()],
            vec![
                distribution_address.simple_serialize().unwrap(),
                index.simple_serialize().unwrap(),
                account.simple_serialize().unwrap(),
                amount.simple_serialize().unwrap(),
                proofs.simple_serialize().unwrap(),
            ],
        );

        let result = association_execute(
            &net,
            &chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
        let status = result.status().status().unwrap();
        // INVALID_PROOF
        assert_eq!(Some(511), move_abort_code(status));
    }

    // claim ok.
    {
        let association_proof = proofs
            .iter()
            .find(|p| p.address == *assocation.address())
            .unwrap();

        let distribution_address = MoveValue::Address(*assocation.address());
        let index = MoveValue::U64(association_proof.index);
        let account = MoveValue::Address(*assocation.address());
        let amount = MoveValue::U128(association_proof.amount);
        let proofs = MoveValue::Vector(
            association_proof
                .proof
                .iter()
                .map(|p| {
                    hex::decode(p.as_str().strip_prefix("0x").unwrap_or_else(|| p.as_str()))
                        .unwrap()
                })
                .map(MoveValue::vector_u8)
                .collect(),
        );
        let script_function = ScriptFunction::new(
            ModuleId::new(
                association_address(),
                Identifier::new("MerkleDistributorScripts").unwrap(),
            ),
            Identifier::new("claim_for_address").unwrap(),
            vec![stc_type_tag()],
            vec![
                distribution_address.simple_serialize().unwrap(),
                index.simple_serialize().unwrap(),
                account.simple_serialize().unwrap(),
                amount.simple_serialize().unwrap(),
                proofs.simple_serialize().unwrap(),
            ],
        );
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
    }

    // after claim, you cannot claim twice.
    {
        let distribution_address = MoveValue::Address(*assocation.address());
        let index = MoveValue::U64(0);

        let ret = execute_readonly_function(
            &chain_state,
            &ModuleId::new(
                association_address(),
                Identifier::new("MerkleDistributor").unwrap(),
            ),
            &Identifier::new("is_claimed").unwrap(),
            vec![stc_type_tag()],
            vec![
                distribution_address.simple_serialize().unwrap(),
                index.simple_serialize().unwrap(),
            ],
            None,
        )?;
        let is_claimed: bool = bcs_ext::from_bytes(ret[0].as_slice()).unwrap();
        assert!(is_claimed, "should already claimed");
    }
    Ok(())
}
