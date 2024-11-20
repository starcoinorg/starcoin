use log::info;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::{
    event::{EventHandle, EventKey},
    on_chain_resource::BlockMetadata,
    state_view::StateReaderExt,
};
use test_helper::executor::prepare_genesis;

#[stest::test]
fn test_block_metadata_bcs_deserialize() -> anyhow::Result<()> {
    let (chain_state, _net) = prepare_genesis();

    let block_metadata = BlockMetadata {
        number: 0,
        parent_hash: Default::default(),
        author: AccountAddress::ONE,
        uncles: 0,
        parents_hash: vec![],
        new_block_events: EventHandle::new(EventKey::new(1, AccountAddress::ONE), 1),
    };
    let bcs_block_metadata = bcs_ext::to_bytes(&block_metadata)?;
    info!(
        "block_metadata: {:?}, length: {}",
        bcs_block_metadata,
        bcs_block_metadata.len()
    );

    let onchain_data = chain_state.get_resource_type_bytes::<BlockMetadata>(genesis_address())?;
    info!(
        "onchain block_metadata: {:?}, data len: {}",
        onchain_data.to_vec(),
        onchain_data.len()
    );
    let on_chain_block_data =
        bcs_ext::from_bytes::<BlockMetadata>(onchain_data.to_vec().as_slice())?;
    assert_eq!(on_chain_block_data.number, 0);

    assert_eq!(
        chain_state
            .get_resource_type::<BlockMetadata>(genesis_address())?
            .number,
        0
    );

    // let output = bcs_ext::from_bytes::<BlockMetadata>(bcs.as_slice())?;
    // assert_eq!(output.number, block_metadata.number);
    //
    // let data = chain_state.get_resource_type::<BlockMetadata>(genesis_address())?;
    // assert_ne!(data.number, 0);
    // assert!(!block_metadata.number > 0);

    Ok(())
}
