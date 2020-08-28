// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_chain_mock::MockChain;
use starcoin_traits::ChainReader;
use starcoin_vm_types::chain_config::ChainNetwork;

#[stest::test]
fn test_block_chain_head() {
    let mut mock_chain = MockChain::new(&ChainNetwork::TEST).unwrap();
    let times = 10;
    mock_chain.produce_and_apply_times(times).unwrap();
    assert_eq!(mock_chain.head().current_header().number, times);
}
