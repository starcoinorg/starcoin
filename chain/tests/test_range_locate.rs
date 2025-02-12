use anyhow::{bail, format_err};
use proptest::result;
use starcoin_chain::{BlockChain, ChainReader, ChainWriter};
use starcoin_chain_api::{
    range_locate::{
        find_common_header_in_range, get_range_in_location, FindCommonHeader, RangeInPruningPoint,
    },
    ExecutedBlock,
};
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_storage::{block_info::BlockInfoStore, BlockStore};
use starcoin_types::block::Block;
use test_helper::Genesis;

fn create_block(count: u64, chain: &mut MockChain) -> anyhow::Result<Vec<ExecutedBlock>> {
    let mut blocks = Vec::new();
    for i in 0..count {
        let header = chain.produce_and_apply_by_tips(
            chain.head().current_header(),
            vec![chain.head().current_header().id()],
        )?;
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
    let mut mock_chain_local =
        MockChain::new_with_genesis_for_test(net.clone(), genesis.clone(), 3)?;
    let mut mock_chain_remote = MockChain::new_with_genesis_for_test(net, genesis.clone(), 3)?;

    let common_number = 37;
    let blocks = create_block(common_number, &mut mock_chain_local)?;

    assert_eq!(
        common_number,
        mock_chain_local.head().current_header().number()
    );

    blocks.into_iter().try_for_each(|block| {
        mock_chain_remote.apply(block.block.clone())?;
        mock_chain_remote.connect(block)?;
        anyhow::Ok(())
    })?;

    assert_eq!(
        common_number,
        mock_chain_remote.head().current_header().number()
    );

    assert_eq!(
        mock_chain_remote.head().current_header().id(),
        mock_chain_local.head().current_header().id()
    );

    let common_block = mock_chain_local
        .get_storage()
        .get_block_by_hash(mock_chain_local.head().current_header().id())?
        .unwrap();

    // now fork
    let _ = create_block(113, &mut mock_chain_remote)?;
    let _ = create_block(13, &mut mock_chain_local)?;

    let mut found_common_header = None;

    let mut remote_start_id = genesis.block().id();
    let mut remote_end_id = None;

    loop {
        println!(
            "remote_start_id: {:?}, remote_end_id: {:?}",
            remote_start_id, remote_end_id
        );
        let result = match get_range_in_location(
            mock_chain_remote.head(),
            &mock_chain_remote.head().dag(),
            mock_chain_remote.head().get_storage(),
            remote_start_id,
            remote_end_id,
        )? {
            RangeInPruningPoint::NotInSelectedChain => bail!("all are no in selected chain!"),
            RangeInPruningPoint::InSelectedChain(hash_value, hash_values) => hash_values,
        };

        result.iter().for_each(|block_id| {
            let header = mock_chain_remote
                .head()
                .get_storage()
                .get_block_header_by_hash(*block_id)
                .unwrap()
                .unwrap();
            println!(
                "result block id: {:?}, number: {:?}",
                header.id(),
                header.number()
            );
        });

        if result.len() == 1 {
            break;
        }

        let find_result = find_common_header_in_range(&mock_chain_local.head().dag(), &result)
            .map_err(|err| {
                format_err!("failed to find_common_header_in_range, error: {:?}", err)
            })?;

        println!("find common header: {:?}", find_result);

        match find_result {
            FindCommonHeader::AllInRange => {
                found_common_header = Some(result.last().unwrap().clone());
                remote_start_id = result.last().unwrap().clone();
                remote_end_id = None;
            }
            FindCommonHeader::InRange(start_id, end_id) => {
                found_common_header = Some(start_id);
                remote_start_id = start_id;
                remote_end_id = Some(end_id);
            }
            FindCommonHeader::Found(hash_value) => {
                found_common_header = Some(hash_value);
                break;
            }
            FindCommonHeader::NotInRange => break,
        }
    }

    println!("found common header: {:?}", found_common_header);

    assert_eq!(
        common_block.id(),
        found_common_header.expect("common header not found")
    );

    anyhow::Ok(())
}
