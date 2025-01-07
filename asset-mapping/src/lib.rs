use anyhow::Result;
use starcoin_crypto::{
    hash::PlainCryptoHash,
    HashValue
};

use forkable_jellyfish_merkle::{
    blob::Blob,
    node_type::SparseMerkleLeafNode,
    RawKey
};
use starcoin_config::ChainNetwork;
use starcoin_state_api::ChainStateReader;
use starcoin_types::{
    account_config::CORE_CODE_ADDRESS,
    identifier::Identifier,
    language_storage::{
        StructTag,
        TypeTag
    },
};
use starcoin_vm_types::{
    access_path::AccessPath,
    account_config::genesis_address,
    genesis_config::ChainId,
    state_store::state_key::StateKey,
    state_view::StateReaderExt
};
use test_helper::executor::{prepare_genesis};

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
        check_blob_hash,
        blob_hash,
        "Check not equal with crypto_hash from Blob"
    );

    Ok(())
}

#[test]
fn test_asset_mapping_for_specified_coin_type() -> Result<()> {
    let (_chain_state, _net) = prepare_genesis();
    let chain_id_struct_tag = StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new("coin").unwrap(),
        name: Identifier::new("CoinStore").unwrap(),
        type_args: vec![TypeTag::Struct(Box::new(StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("starcoin_coin").unwrap(),
            name: Identifier::new("STC").unwrap(),
            type_args: vec![],
        }))],
    };

    let access_path = AccessPath::resource_access_path(genesis_address(), chain_id_struct_tag);
    let (account_address, data_path) = access_path.into_inner();

    println!(
        "test_asset_mapping_for_specified_coin_type | account {:?}, data_path: {:?}, data_path key hash: {:?}",
        account_address,
        data_path.encode_key()?,
        data_path.key_hash()
    );

    Ok(())
}

