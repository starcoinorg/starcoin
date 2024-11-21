use log::info;
use starcoin_crypto::HashValue;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::{
    event::{EventHandle},
    on_chain_resource::BlockMetadataV2,
    state_view::StateReaderExt,
};
use test_helper::executor::prepare_genesis;

#[stest::test]
fn test_block_metadata_bcs_deserialize() -> anyhow::Result<()> {
    let (chain_state, _net) = prepare_genesis();

    let block_metadata = BlockMetadataV2 {
        number: 0,
        parent_hash: HashValue::sha3_256_of(b"starcoin_test"),
        author: AccountAddress::ONE,
        uncles: 0,
        parents_hash: vec![],
        new_block_events: EventHandle::new_from_address(&AccountAddress::ZERO, 1),
    };
    println!("{:?}", block_metadata);
    let bcs_block_metadata = bcs_ext::to_bytes(&block_metadata)?;
    info!(
        "block_metadata: {:?}, length: {}",
        bcs_block_metadata,
        bcs_block_metadata.len()
    );

    let onchain_data = chain_state.get_resource::<BlockMetadataV2>(genesis_address())?.unwrap();
    println!("onchain_data {:?}", onchain_data);
    assert_eq!(block_metadata.parent_hash, onchain_data.parent_hash);

    Ok(())
}


