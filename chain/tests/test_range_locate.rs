use starcoin_chain::{BlockChain, ChainReader, ChainWriter};
use starcoin_chain_api::ExecutedBlock;
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_storage::{block_info::BlockInfoStore, BlockStore};
use starcoin_types::block::Block;
use test_helper::Genesis;

fn create_block(count: u64, chain: &mut MockChain) -> anyhow::Result<Vec<ExecutedBlock>> {
    let mut blocks = Vec::new();
    for i in 0..count {
        let header = chain.produce_and_apply_by_tips(chain.head().current_header(), vec![chain.head().current_header().id()])?;
        let block = chain.get_storage().get_block_by_hash(header.id())?.unwrap();
        let block_info = chain.get_storage().get_block_info(header.id())?.unwrap();
        let executed_block = ExecutedBlock::new(block, block_info);
        chain.connect(executed_block.clone())?;
        blocks.push(executed_block);
    }
    Ok(blocks)
}

#[stest::test]
fn test_range_locate() -> anyhow::Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain_local = MockChain::new_with_genesis_for_test(net.clone(), genesis.clone(), 3)?;
    let mut mock_chain_remote = MockChain::new_with_genesis_for_test(net, genesis, 3)?;

    let common_number = 37;
    let blocks = create_block(common_number, &mut mock_chain_local)?;

    assert_eq!(common_number, mock_chain_local.head().current_header().number());

    blocks.into_iter().try_for_each(|block| {
        mock_chain_remote.apply(block.block.clone())?;
        mock_chain_remote.connect(block)?;
        anyhow::Ok(())
    })?; 

    assert_eq!(common_number, mock_chain_remote.head().current_header().number());

    anyhow::Ok(())
}