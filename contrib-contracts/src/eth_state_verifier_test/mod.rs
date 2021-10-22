use anyhow::Result;
use ethereum_types::{BigEndianHash, H160, H256, U256};
use rlp::Encodable;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};
use starcoin_executor::execute_readonly_function;
use starcoin_types::account_config::association_address;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::TransactionPayload;
use starcoin_vm_types::transaction::Package;
use starcoin_vm_types::value::MoveValue;
use test_helper::executor::{
    association_execute_should_success, compile_modules_with_address, prepare_genesis,
};

/// Basic account type.
#[derive(Debug, Clone, PartialEq, Eq, RlpEncodable, RlpDecodable)]
pub struct BasicAccount {
    /// Nonce of the account.
    pub nonce: U256,
    /// Balance of the account.
    pub balance: U256,
    /// Storage root of the account.
    pub storage_root: H256,
    /// Code hash of the account.
    pub code_hash: H256,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthAccount {
    pub state_root: H256,
    pub height: u64,
    pub address: H160,
    pub balance: U256,
    pub nonce: U256,
    pub code_hash: H256,
    pub storage_hash: H256,
    pub account_proof: Vec<String>,
    pub storage_proof: Vec<StorageProof>,
}

impl EthAccount {
    pub fn account_value(&self) -> BasicAccount {
        BasicAccount {
            nonce: self.nonce,
            balance: self.balance,
            storage_root: self.storage_hash,
            code_hash: self.code_hash,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StorageProof {
    pub key: U256,
    pub value: U256,
    pub proof: Vec<String>,
}

#[stest::test]
fn test_eth_state_proof_verify() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    // deploy the module
    {
        let source = include_str!("../../modules/EthStateVerifier.move");
        let modules = compile_modules_with_address(association_address(), source);

        let package = Package::new(modules, None)?;
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::Package(package),
        )?;
    }

    // load the example proof
    let account_proof: EthAccount = {
        let proofs = include_str!("proof.json");
        let value: serde_json::Value = serde_json::from_str(proofs)?;
        serde_json::from_value(value)?
    };

    // verify account proof
    {
        let account_in_rlp = rlp::encode(&account_proof.account_value());
        let proofs: Vec<_> = account_proof
            .account_proof
            .iter()
            .map(|p| hex::decode(&p.as_str()[2..]).unwrap())
            .collect();

        let expected_state_root = account_proof.state_root;
        let expected_state_root = MoveValue::vector_u8(expected_state_root.as_bytes().to_vec());

        let key = {
            let key = account_proof.address.as_bytes().to_vec();
            MoveValue::vector_u8(key)
        };

        let proof = MoveValue::Vector(
            proofs
                .iter()
                .map(|p| MoveValue::vector_u8(p.clone()))
                .collect(),
        );
        let expected_value = MoveValue::vector_u8(account_in_rlp);

        let result = execute_readonly_function(
            &chain_state,
            &ModuleId::new(
                association_address(),
                Identifier::new("EthStateVerifier").unwrap(),
            ),
            &Identifier::new("verify").unwrap(),
            vec![],
            vec![
                expected_state_root.simple_serialize().unwrap(),
                key.simple_serialize().unwrap(),
                proof.simple_serialize().unwrap(),
                expected_value.simple_serialize().unwrap(),
            ],
            None,
        )?
        .pop()
        .unwrap();

        let is_ok: bool = bcs_ext::from_bytes(result.as_slice()).unwrap();
        assert!(is_ok, "verify account fail");
    }

    // verify storage proof
    {
        for storage_proof in &account_proof.storage_proof {
            println!("proof: {:?}", storage_proof);
            let key = H256::from_uint(&storage_proof.key).as_bytes().to_vec();
            let value = storage_proof.value.rlp_bytes();

            let proofs: Vec<_> = storage_proof
                .proof
                .iter()
                .map(|p| hex::decode(&p.as_str()[2..]).unwrap())
                .collect();

            let expected_state_root = account_proof.storage_hash;
            let expected_state_root = MoveValue::vector_u8(expected_state_root.as_bytes().to_vec());
            let key = { MoveValue::vector_u8(key) };
            let proof = MoveValue::Vector(
                proofs
                    .iter()
                    .map(|p| MoveValue::vector_u8(p.clone()))
                    .collect(),
            );
            let expected_value = MoveValue::vector_u8(value);

            let result = execute_readonly_function(
                &chain_state,
                &ModuleId::new(
                    association_address(),
                    Identifier::new("EthStateVerifier").unwrap(),
                ),
                &Identifier::new("verify").unwrap(),
                vec![],
                vec![
                    expected_state_root.simple_serialize().unwrap(),
                    key.simple_serialize().unwrap(),
                    proof.simple_serialize().unwrap(),
                    expected_value.simple_serialize().unwrap(),
                ],
                None,
            )?
            .pop()
            .unwrap();

            let is_ok: bool = bcs_ext::from_bytes(result.as_slice()).unwrap();
            assert!(is_ok, "verify storage proof fail");
        }
    }

    Ok(())
}
