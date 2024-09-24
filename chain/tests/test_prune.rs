use std::collections::HashSet;

use rand::rngs::mock;
use starcoin_chain::ChainReader;
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::debug;




#[stest::test]
fn test_block_chain_prune() -> anyhow::Result<()> {
    let mut mock_chain = MockChain::new_with_params(ChainNetwork::new_test(), 3, 4, 3)?;
    let genesis = mock_chain.head().status().head.clone();

    // blue blocks
    let block_blue_1 = mock_chain.produce_block_by_tips(genesis.clone(), vec![genesis.id()])?;
    mock_chain.apply(block_blue_1.clone())?;
    let block_blue_2 = mock_chain.produce_block_by_tips(block_blue_1.header().clone(), vec![block_blue_1.id()])?;
    mock_chain.apply(block_blue_2.clone())?;
    let block_blue_3 = mock_chain.produce_block_by_tips(block_blue_2.header().clone(), vec![block_blue_2.id()])?;
    mock_chain.apply(block_blue_3.clone())?;
    let block_blue_3_1 = mock_chain.produce_block_by_tips(block_blue_2.header().clone(), vec![block_blue_2.id()])?;
    mock_chain.apply(block_blue_3_1.clone())?;
    let block_blue_4 = mock_chain.produce_block_by_tips(block_blue_3.header().clone(), vec![block_blue_3.id(), block_blue_3_1.id()])?;
    mock_chain.apply(block_blue_4.clone())?;
    let block_blue_5 = mock_chain.produce_block_by_tips(block_blue_4.header().clone(), vec![block_blue_4.id()])?;
    mock_chain.apply(block_blue_5.clone())?;

    // red blocks
    let block_red_2 = mock_chain.produce_block_by_tips(block_blue_1.header().clone(), vec![block_blue_1.id()])?;
    mock_chain.apply(block_red_2.clone())?;
    let block_red_2_1 = mock_chain.produce_block_by_tips(block_blue_1.header().clone(), vec![block_blue_1.id()])?;
    mock_chain.apply(block_red_2_1.clone())?;
    let block_red_3 = mock_chain.produce_block_by_tips(block_red_2.header().clone(), vec![block_red_2.id(), block_red_2_1.id()])?;
    mock_chain.apply(block_red_3.clone())?;

    debug!("tips: {:?}, pruning point: {:?}", mock_chain.head().get_dag_state()?, mock_chain.head().status().head().pruning_point());
    assert_eq!(mock_chain.head().get_dag_state()?.tips.into_iter().collect::<HashSet<_>>(), HashSet::from_iter(vec![block_blue_5.id(), block_red_3.id()]));



    Ok(())
}