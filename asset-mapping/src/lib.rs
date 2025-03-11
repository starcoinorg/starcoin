#[cfg(test)]
mod basic_tests {
    use std::collections::BTreeMap;
    use std::str::FromStr;

    use anyhow::Result;
    use starcoin_crypto::{hash::PlainCryptoHash, HashValue};

    use bytes::Bytes;
    use forkable_jellyfish_merkle::{blob::Blob, node_type::SparseMerkleLeafNode, RawKey};
    use move_core_types::language_storage::StructTag as MoveStructTag;
    use move_core_types::move_resource::MoveStructType;
    use starcoin_cached_packages::starcoin_framework_sdk_builder::{
        asset_mapping_assign_to_account_test, asset_mapping_assign_to_account_with_proof,
    };
    use starcoin_cached_packages::starcoin_stdlib::{
        starcoin_account_create_account, transfer_scripts_peer_to_peer_v2,
    };
    use starcoin_config::{ChainNetwork, G_TEST_CONFIG};
    use starcoin_logger::prelude::warn;
    use starcoin_state_api::ChainStateReader;
    use starcoin_statedb::ChainStateDB;
    use starcoin_types::{
        account_address::AccountAddress, account_config::CORE_CODE_ADDRESS, identifier::Identifier,
        language_storage::StructTag,
    };
    use starcoin_vm_types::account_config::{
        fungible_store, stc_struct_tag, CoinStoreResource, FungibleStoreResource,
        ObjectGroupResource, G_STC_TOKEN_CODE,
    };
    use starcoin_vm_types::state_store::TStateView;
    use starcoin_vm_types::{
        access_path::AccessPath,
        account_config::{genesis_address, stc_type_tag},
        genesis_config::ChainId,
        state_store::state_key::StateKey,
        state_view::StateReaderExt,
    };
    use test_helper::executor::{
        association_execute_should_success, prepare_customized_genesis, prepare_genesis,
    };

    fn get_resource_from_group(
        statedb: &ChainStateDB,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> Result<Option<Bytes>> {
        Ok(
            if let Some(group_data_blob) = statedb.get_state_value_bytes(group_key)? {
                let group_data =
                    bcs_ext::from_bytes::<BTreeMap<StructTag, Bytes>>(&group_data_blob)?;
                group_data.get(resource_tag).cloned()
            } else {
                None
            },
        )
    }

    fn read_starcoin_balance(statedb: &ChainStateDB, address: AccountAddress) -> Result<u128> {
        let type_tag: MoveStructTag = G_STC_TOKEN_CODE.clone().try_into()?;
        // Get from coin store
        let coin_balance = match statedb.get_state_value_bytes(&StateKey::resource(
            &address,
            &CoinStoreResource::struct_tag_for_token(type_tag.clone()),
        )?)? {
            Some(bytes) => bcs_ext::from_bytes::<CoinStoreResource>(&bytes)?.coin() as u128,
            None => {
                warn!(
                    "CoinStoreResource not exists at address:{} for type tag:{}",
                    address, type_tag
                );
                0
            }
        };

        let primary_store_address = fungible_store::primary_store(&address, &type_tag.address);
        // Get from coin store
        let object_group_tag = ObjectGroupResource::struct_tag();
        let fungible_store_tag = FungibleStoreResource::struct_tag();
        let fungible_store_group_key =
            StateKey::resource_group(&primary_store_address, &object_group_tag);

        let fungible_balance =
            match get_resource_from_group(statedb, &fungible_store_group_key, &fungible_store_tag)?
            {
                Some(bytes) => {
                    bcs_ext::from_bytes::<FungibleStoreResource>(&bytes)?.balance() as u128
                }
                None => {
                    warn!(
                        "FungibleStoreResource not exists at address:{:?} for type tag:{:?}",
                        primary_store_address, fungible_store_group_key
                    );
                    0
                }
            };
        Ok(coin_balance + fungible_balance)
    }

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
        starcoin_logger::init_for_test();

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
        starcoin_logger::init_for_test();
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
    fn test_simple_asset_mapping_without_proof() -> Result<()> {
        starcoin_logger::init_for_test();

        let (chain_state, net) = prepare_genesis();

        // Build alice and transfer STC to alice
        let amount = 1000000000;
        let alice = AccountAddress::from_str("0xd0c5a06ae6100ce115cad1600fe59e96").unwrap();

        association_execute_should_success(
            &net,
            &chain_state,
            transfer_scripts_peer_to_peer_v2(stc_type_tag(), alice, amount),
        )?;

        // Check alice's balance
        assert_eq!(
            read_starcoin_balance(&chain_state, alice)?,
            amount,
            "alice balance not expect"
        );

        association_execute_should_success(
            &net,
            &chain_state,
            asset_mapping_assign_to_account_test(
                alice,
                "0x1::STC::STC".as_bytes().to_vec(),
                amount as u64,
            ),
        )?;

        assert_eq!(
            read_starcoin_balance(&chain_state, alice)?,
            amount * 2,
            "alice balance not expect after asset-mapping"
        );

        Ok(())
    }

    #[test]
    fn  s() -> Result<()> {
        starcoin_logger::init_for_test();

        // let _block_gas_limit: u64 = 10000000;
        let initial_balance: u128 = 100000000000; // 1000 STC
        let alice = AccountAddress::from_str("0xd0c5a06ae6100ce115cad1600fe59e96").unwrap();

        // Create a source BlockChain
        let (
            proof_root_hash,
            proof_path_hash,
            proof_value_hash,
            proof_siblings,
            res_root_hash,
            res_path_hash,
            res_value_hash,
            res_siblings,
        ) = {
            let (chain_state_1, net_1) = prepare_genesis();

            association_execute_should_success(
                &net_1,
                &chain_state_1,
                transfer_scripts_peer_to_peer_v2(stc_type_tag(), alice, initial_balance),
            )?;

            // Check balance is initial_balance
            let balance = read_starcoin_balance(&chain_state_1, alice)?;
            assert_eq!(balance, initial_balance);

            let state_proof = chain_state_1.get_with_proof(&StateKey::resource(
                &alice,
                &CoinStoreResource::struct_tag_for_token(stc_struct_tag()),
            )?)?;
            let account_state = chain_state_1.get_account_state(&alice)?;
            let resource_root = account_state.resource_root();

            (
                chain_state_1.state_root(),
                state_proof.proof.account_proof.leaf().unwrap().0,
                state_proof.proof.account_proof.leaf().unwrap().1,
                state_proof.proof.account_proof.siblings,
                resource_root,
                state_proof.proof.account_state_proof.leaf().unwrap().0,
                state_proof.proof.account_state_proof.leaf().unwrap().1,
                state_proof.proof.account_state_proof.siblings,
            )
        };

        println!(
            "test_asset_mapping_whole_process | proof_root_hash: {:?}, proof_path_hash: {:?}, proof_value_hash: {:?}, proof_siblings: {:?}",
            proof_root_hash, proof_path_hash, proof_value_hash, proof_siblings,
        );

        // Create a new blockchain and verify proof
        {
            let custom_chain_id = ChainId::new(100);
            let mut genesis_config = G_TEST_CONFIG.clone();
            genesis_config.asset_mapping_root_hash = proof_root_hash;

            let net = ChainNetwork::new_custom(
                "asset_mapping_test".parse()?,
                custom_chain_id,
                genesis_config,
            )?;

            let chain_state_2 = prepare_customized_genesis(&net);

            let mut proof_encoded_siblings: Vec<u8> = Vec::new();
            proof_siblings.iter().for_each(|hash| {
                proof_encoded_siblings.extend_from_slice(hash.as_ref());
                proof_encoded_siblings.push(0x7c);
            });

            let mut res_encoded_siblings: Vec<u8> = Vec::new();
            res_siblings.iter().for_each(|hash| {
                res_encoded_siblings.extend_from_slice(hash.as_ref());
                res_encoded_siblings.push(0x7c);
            });

            // Create account
            association_execute_should_success(
                &net,
                &chain_state_2,
                starcoin_account_create_account(alice),
            )?;

            assert_eq!(read_starcoin_balance(&chain_state_2, alice)?, 0);

            // Asset mapping for alice
            association_execute_should_success(
                &net,
                &chain_state_2,
                asset_mapping_assign_to_account_with_proof(
                    alice,
                    "0x1::STC::STC".as_bytes().to_vec(),
                    proof_path_hash.to_vec(),
                    proof_value_hash.to_vec(),
                    proof_encoded_siblings,
                    res_root_hash.to_vec(),
                    res_path_hash.to_vec(),
                    res_value_hash.to_vec(),
                    res_encoded_siblings,
                    initial_balance as u64,
                ),
            )?;

            assert_eq!(
                read_starcoin_balance(&chain_state_2, alice)?,
                initial_balance
            );
        }

        Ok(())
    }

    #[test]
    fn test_hash_hello() -> Result<()> {
        // 0x3338be694f50c5f338814986cdf0686453a888b84f424d792af4b9202398f392
        let hello_hash = HashValue::sha3_256_of("hello".as_bytes());
        println!("test_hash_hello | {:?}", hello_hash);
        assert_eq!(
            hello_hash.to_hex_literal(),
            "0x3338be694f50c5f338814986cdf0686453a888b84f424d792af4b9202398f392",
            "not expect hash"
        );
        Ok(())
    }
}
