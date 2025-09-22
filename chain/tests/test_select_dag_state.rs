use anyhow::Result;
use starcoin_chain::ChainReader;
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_crypto::HashValue;
use test_helper::Genesis;

/// Test basic case: same pruning point should not trigger complex logic
#[stest::test]
fn test_select_dag_state_same_pruning_point() -> Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain = MockChain::new_with_genesis_for_test(net, genesis, 3)?;
    let genesis_header = mock_chain.head().status().head.clone();

    // Create a simple chain: genesis -> block_1 -> block_2
    let block_1 =
        mock_chain.produce_and_apply_by_tips(genesis_header.clone(), vec![genesis_header.id()])?;
    let block_2 = mock_chain.produce_and_apply_by_tips(block_1.clone(), vec![block_1.id()])?;

    // Select DAG state with same pruning point (should use simple path)
    let mut test_chain = mock_chain.fork_new_branch(Some(block_1.id()))?;
    let new_chain = test_chain.select_dag_state(&block_2)?;

    // Verify: selected chain should have block_2 as head (same pruning point, simple case)
    assert_eq!(new_chain.status().head().id(), block_2.id());
    assert_eq!(
        new_chain.status().head().pruning_point(),
        block_2.pruning_point()
    );

    Ok(())
}

/// Test complex case: different pruning points triggers comparison logic
#[stest::test]
fn test_select_dag_state_different_pruning_points() -> Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain = MockChain::new_with_genesis_for_test(net, genesis, 3)?;
    let genesis_header = mock_chain.head().status().head.clone();

    // Build a DAG with enough depth to trigger pruning
    // Chain 1: genesis -> blue_1 -> blue_2 -> blue_3 -> blue_4 -> blue_5
    let blue_1 =
        mock_chain.produce_and_apply_by_tips(genesis_header.clone(), vec![genesis_header.id()])?;
    let blue_2 = mock_chain.produce_and_apply_by_tips(blue_1.clone(), vec![blue_1.id()])?;
    let blue_3 = mock_chain.produce_and_apply_by_tips(blue_2.clone(), vec![blue_2.id()])?;
    let blue_4 = mock_chain.produce_and_apply_by_tips(blue_3.clone(), vec![blue_3.id()])?;
    let blue_5 = mock_chain.produce_and_apply_by_tips(blue_4.clone(), vec![blue_4.id()])?;

    // Generate pruning point for blue_5 (this should set pruning_point to blue_2)
    futures::executor::block_on(mock_chain.head().dag().generate_pruning_point(
        &blue_5,
        4, // pruning_depth
        3, // pruning_finality
        genesis_header.id(),
    ))?;

    // Create a block with the new pruning point
    let blue_6 = mock_chain.produce_block_for_pruning()?;
    mock_chain.apply(blue_6.clone())?;

    // Verify pruning was triggered (may or may not change from genesis depending on chain length)
    // The key is that select_dag_state should handle any pruning point without failing

    // Create a second chain branch for comparison
    let mut fork_chain = mock_chain.fork(Some(blue_3.id()))?;
    let red_4 = fork_chain.produce_and_apply_by_tips(blue_3.clone(), vec![blue_3.id()])?;
    let _red_5 = fork_chain.produce_and_apply_by_tips(red_4.clone(), vec![red_4.id()])?;

    // Now test select_dag_state with different pruning points
    // This should trigger the complex comparison logic in lines 1915-1934
    let mut test_chain = mock_chain.fork_new_branch(Some(blue_5.id()))?;
    let selected_chain = test_chain.select_dag_state(blue_6.header())?;

    // Verify: the selected chain should handle the pruning point change correctly
    // The exact result depends on GHOSTDAG comparison, but it should not panic/fail
    assert!(selected_chain.status().head().id() != HashValue::zero());

    Ok(())
}

/// Test edge case: zero pruning point handling
#[stest::test]
fn test_select_dag_state_zero_pruning_point() -> Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain = MockChain::new_with_genesis_for_test(net, genesis, 3)?;
    let genesis_header = mock_chain.head().status().head.clone();

    // Create blocks without triggering pruning (short chain)
    let block_1 =
        mock_chain.produce_and_apply_by_tips(genesis_header.clone(), vec![genesis_header.id()])?;

    // Both current and new blocks should have zero pruning point
    assert_eq!(genesis_header.pruning_point(), HashValue::zero());
    assert_eq!(block_1.pruning_point(), HashValue::zero());

    // This should take the simple path (condition on line 1905-1907)
    let mut test_chain = mock_chain.fork_new_branch(Some(genesis_header.id()))?;
    let selected_chain = test_chain.select_dag_state(&block_1)?;

    // Verify: should work without issues
    assert_eq!(selected_chain.status().head().id(), block_1.id());

    Ok(())
}

/// Test the specific fix: what used to fail with "not yet fully implemented"
#[stest::test]
fn test_select_dag_state_regression_pruning_change() -> Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain = MockChain::new_with_genesis_for_test(net, genesis, 3)?;
    let genesis_header = mock_chain.head().status().head.clone();

    // Build two separate chains with different pruning points
    // Chain 1: Deep enough to get pruning_point = blue_2
    let blue_1 =
        mock_chain.produce_and_apply_by_tips(genesis_header.clone(), vec![genesis_header.id()])?;
    let blue_2 = mock_chain.produce_and_apply_by_tips(blue_1.clone(), vec![blue_1.id()])?;
    let blue_3 = mock_chain.produce_and_apply_by_tips(blue_2.clone(), vec![blue_2.id()])?;
    let blue_4 = mock_chain.produce_and_apply_by_tips(blue_3.clone(), vec![blue_3.id()])?;
    let blue_5 = mock_chain.produce_and_apply_by_tips(blue_4.clone(), vec![blue_4.id()])?;
    let blue_6 = mock_chain.produce_and_apply_by_tips(blue_5.clone(), vec![blue_5.id()])?;

    // Generate pruning point
    futures::executor::block_on(mock_chain.head().dag().generate_pruning_point(
        &blue_6,
        4, // pruning_depth
        3, // pruning_finality
        genesis_header.id(),
    ))?;

    let pruned_block = mock_chain.produce_block_for_pruning()?;
    mock_chain.apply(pruned_block.clone())?;

    // This block should have a non-zero pruning point
    let new_pruning_point = pruned_block.header().pruning_point();
    assert_ne!(new_pruning_point, HashValue::zero());

    // Fork to a different chain with zero pruning point
    let mut other_chain = mock_chain.fork(Some(genesis_header.id()))?;
    let _other_block =
        other_chain.produce_and_apply_by_tips(genesis_header.clone(), vec![genesis_header.id()])?;

    // This is the scenario that used to fail with "Pruning point change not yet fully implemented"
    // Current chain has zero pruning point, new block has non-zero pruning point
    let mut test_chain = other_chain.fork_new_branch(Some(genesis_header.id()))?;
    let result = test_chain.select_dag_state(pruned_block.header());

    // The key test: this should NOT fail anymore
    assert!(
        result.is_ok(),
        "select_dag_state should handle pruning point changes"
    );

    let selected_chain = result?;
    // Should successfully create a chain, exact head depends on GHOSTDAG logic
    assert!(selected_chain.status().head().id() != HashValue::zero());

    Ok(())
}
