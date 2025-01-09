use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};

use forkable_jellyfish_merkle::{blob::Blob, node_type::SparseMerkleLeafNode, RawKey};
use starcoin_cached_packages::starcoin_framework_sdk_builder::asset_mapping_assign_to_account_with_proof;
use starcoin_chain::{ChainReader, ChainWriter};
use starcoin_config::{ChainNetwork, NodeConfig, G_TEST_CONFIG};
use starcoin_consensus::Consensus;
use starcoin_state_api::ChainStateReader;
use starcoin_transaction_builder::DEFAULT_MAX_GAS_AMOUNT;
use starcoin_types::{
    account::DEFAULT_EXPIRATION_TIME, account_address::AccountAddress,
    account_config::CORE_CODE_ADDRESS, identifier::Identifier, language_storage::StructTag,
};
use starcoin_types::account::Account;
use starcoin_vm_types::account_config::stc_type_tag;
use starcoin_vm_types::{
    access_path::AccessPath, account_config, account_config::genesis_address,
    genesis_config::ChainId, state_store::state_key::StateKey, state_view::StateReaderExt,
};
use starcoin_vm_types::transaction::TransactionPayload;
use test_helper::executor::prepare_genesis;

#[test]
fn test_get_chain_id_after_genesis_with_proof_verify() -> Result<()> {
    let (chain_state, _) = prepare_genesis();
    let chain_id_struct_tag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("chain_id").unwrap(),
        name: Identifier::new("ChainId").unwrap(),
        type_args: vec![],
    };

    let path_statekey = StateKey::resource(&CORE_CODE_ADDRESS, &chain_id_struct_tag)?;

    // Print 0x1 version resource
    let resource_value = bcs_ext::from_bytes::<ChainId>(
        &chain_state.get_resource(CORE_CODE_ADDRESS, &chain_id_struct_tag)?,
    )?;
    println!(
        "test_get_chain_id_after_genesis_with_proof_verify | path: {:?}, state_value : {:?}",
        chain_id_struct_tag, resource_value
    );
    assert_eq!(resource_value.id(), 0xff, "not expect chain id");

    // Get proof and verify proof
    let mut state_proof = chain_state.get_with_proof(&path_statekey)?;
    let proof_path = AccessPath::resource_access_path(genesis_address(), chain_id_struct_tag);
    state_proof.verify(chain_state.state_root(), proof_path.clone())?;

    state_proof.state.as_mut().unwrap()[0] = 0xFE;
    assert!(state_proof
        .verify(chain_state.state_root(), proof_path)
        .is_err());
    Ok(())
}

#[test]
fn test_sha3_256_diffrent_with_crypto_macro() -> Result<()> {
    let element_key_hash = HashValue::from_hex_literal(
        "0x4cc8bd9df94b37c233555d9a3bba0a712c3c709f047486d1e624b2bcd3b83266",
    )?;
    let blob_hash = HashValue::from_hex_literal(
        "0x4f2b59b9af93b435e0a33b6ab7a8a90e471dba936be2bc2937629b7782b8ebd0",
    )?;

    let leaf_node = SparseMerkleLeafNode::new(element_key_hash, blob_hash);

    let smt_hash = leaf_node.crypto_hash();
    println!(
        "test_sha3_256_diffrent_with_crypto_macro | SparseMerkleLeafNode crypto hash: {:?}",
        SparseMerkleLeafNode::new(element_key_hash, blob_hash).crypto_hash()
    );

    let ser = bcs_ext::to_bytes(&leaf_node)?;
    const STARCOIN_HASH_PREFIX: &[u8] = b"STARCOIN::SparseMerkleLeafNode";
    let hash_vec = [
        HashValue::sha3_256_of(STARCOIN_HASH_PREFIX).as_slice(),
        ser.as_slice(),
    ]
    .concat();

    let move_hash = HashValue::sha3_256_of(&hash_vec[..]);
    println!(
        "test_sha3_256_diffrent_with_crypto_macro | sha3 crypto {:?}",
        move_hash,
    );
    assert_eq!(move_hash, smt_hash, "Failed to get the same hash");

    let check_blob_hash = Blob::from(Vec::from([255])).crypto_hash();
    assert_eq!(
        check_blob_hash, blob_hash,
        "Check not equal with crypto_hash from Blob"
    );

    Ok(())
}

#[test]
fn test_asset_mapping_for_specified_coin_type() -> Result<()> {
    let (_chain_state, _net) = prepare_genesis();
    let stc_store_tag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("coin").unwrap(),
        name: Identifier::new("CoinStore").unwrap(),
        type_args: vec![stc_type_tag()],
    };

    let access_path = AccessPath::resource_access_path(genesis_address(), stc_store_tag);
    let (account_address, data_path) = access_path.into_inner();

    println!(
        "test_asset_mapping_for_specified_coin_type | account {:?}, data_path: {:?}, data_path key hash: {:?}",
        account_address,
        data_path.encode_key()?,
        data_path.key_hash()
    );
    Ok(())
}

#[test]
fn test_asset_mapping_whole_process() -> Result<()> {
    let block_gas_limit: u64 = 10000000;
    let initial_balance: u128 = 1000000000000;
    let receiver = AccountAddress::from_str("0xd0c5a06ae6100ce115cad1600fe59e96").unwrap();

    // Create a source BlockChain
    let (proof_root_hash, proof_path_hash, proof_value_hash, proof_siblings) = {
        // construct a test network
        let config = Arc::new(NodeConfig::random_for_test());
        let mut block_chain = test_helper::gen_blockchain_with_blocks_for_test(0, config.net())?;

        let association_sequence_num = {
            block_chain
                .chain_state_reader()
                .get_sequence_number(account_config::association_address())?
        };

        let transaction_txn = starcoin_transaction_builder::build_transfer_from_association(
            receiver,
            association_sequence_num,
            initial_balance,
            config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            config.net(),
        )
        .try_into()?;

        let (block_template, _excluded) = block_chain
            .create_block_template(
                account_config::association_address(),
                None,
                vec![transaction_txn],
                vec![],
                Some(block_gas_limit),
                None,
            )
            .unwrap();

        let block = block_chain
            .consensus()
            .create_block(block_template, block_chain.time_service().as_ref())?;

        block_chain.apply(block)?;

        {
            let chain_state = block_chain.chain_state_reader();

            // Check balance is initial_balance
            let balance = block_chain.chain_state_reader().get_balance(receiver)?;
            assert_eq!(balance, initial_balance);

            let proof = chain_state.get_with_proof(&StateKey::resource(
                &CORE_CODE_ADDRESS,
                &StructTag {
                    address: CORE_CODE_ADDRESS,
                    module: Identifier::new("coin").unwrap(),
                    name: Identifier::new("CoinStore").unwrap(),
                    type_args: vec![stc_type_tag()],
                },
            )?)?;
            (
                chain_state.state_root(),
                proof.proof.account_state_proof.leaf.unwrap().0,
                proof.proof.account_state_proof.leaf.unwrap().1,
                proof.proof.account_state_proof.siblings,
            )
        }
    };

    println!(
        "test_asset_mapping_whole_process | proof_root_hash: {:?}, proof_path_hash: {:?}, proof_value_hash: {:?}, proof_siblings: {:?}",
        proof_root_hash, proof_path_hash, proof_value_hash, proof_siblings,
    );

    {
        let custom_chain_id = ChainId::new(100);
        let mut genesis_config = G_TEST_CONFIG.clone();
        genesis_config.asset_mapping_root_hash = proof_root_hash;
        let net = ChainNetwork::new_custom("asset_mapping_test".parse()?, custom_chain_id, genesis_config)?;
        let mut block_chain = test_helper::gen_blockchain_with_blocks_for_test(0, &net)?;

        let mut proof_encoded_siblings: Vec<u8> = Vec::new();
        proof_siblings.iter().for_each(|hash| {
            proof_encoded_siblings.extend_from_slice(hash.as_ref());
            proof_encoded_siblings.push(0x7c);
        });

        let genesis_sequence_number = {
            block_chain
                .chain_state_reader()
                .get_sequence_number(genesis_address())?
        };

        let asset_mapping_txn =  Account::new_genesis_account(genesis_address()).create_signed_txn_with_args(
            asset_mapping_assign_to_account_with_proof(
                receiver,
                "0x1::STC::STC".as_bytes().to_vec(),
                proof_path_hash.to_vec(),
                proof_value_hash.to_vec(),
                proof_encoded_siblings,
                initial_balance as u64,
            ),
            genesis_sequence_number,
            DEFAULT_MAX_GAS_AMOUNT,
            block_gas_limit,
            DEFAULT_EXPIRATION_TIME,
            net.chain_id(),
        );

        let (block_template, _excluded) = block_chain
            .create_block_template(
                account_config::association_address(),
                None,
                vec![asset_mapping_txn],
                vec![],
                Some(block_gas_limit),
                None,
            )
            .unwrap();

        block_chain.apply(block_chain
            .consensus()
            .create_block(block_template, block_chain.time_service().as_ref())?)?;

        {
            let balance = block_chain.chain_state_reader().get_balance(receiver)?;
            assert_eq!(balance, initial_balance);
        }
    }

    Ok(())
}

fn execute_block_with_txn() {}
