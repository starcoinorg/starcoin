use starcoin_chain::{BlockChain, ChainReader};
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_types::block::BlockHeader;
use test_helper::Genesis;

fn create_block(count: u64, chain: &mut MockChain) -> anyhow::Result<Vec<BlockHeader>> {
    let mut headers = Vec::new();
    for i in 0..count {
      let header = chain.produce_and_apply_by_tips(chain.head().current_header(), vec![chain.head().current_header().id()])?;
        headers.push(header);
    }
    Ok(headers)
}

fn init_genesis() -> anyhow::Result<> {

}

#[stest::test]
fn test_range_locate() -> anyhow::Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain_local = MockChain::new_with_genesis_for_test(net.clone(), genesis.clone(), 3)?;
    let mut mock_chain_remote = MockChain::new_with_genesis_for_test(net, genesis, 3)?;


    anyhow::Ok(())
}