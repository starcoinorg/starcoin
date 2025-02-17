use anyhow::{bail, format_err};
use starcoin_chain::ChainReader;
use starcoin_chain_api::range_locate::{
    find_common_header_in_range, get_range_in_location, FindCommonHeader, RangeInLocation,
};
use starcoin_chain_mock::MockChain;
use starcoin_config::ChainNetwork;
use starcoin_storage::BlockStore;
use test_helper::Genesis;

#[stest::test]
fn test_range_locate() -> anyhow::Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain_local =
        MockChain::new_with_genesis_for_test(net.clone(), genesis.clone(), 3)?;
    let mut mock_chain_remote = MockChain::new_with_genesis_for_test(net, genesis.clone(), 3)?;

    let common_number = 37;
    let blocks = mock_chain_local.produce_and_apply_with_tips_for_times(common_number)?;

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
    let _ = mock_chain_remote.produce_and_apply_with_tips_for_times(113)?;
    let _ = mock_chain_local.produce_and_apply_with_tips_for_times(13)?;

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
            mock_chain_remote.head().get_storage(),
            remote_start_id,
            remote_end_id,
        )? {
            RangeInLocation::NotInSelectedChain => bail!("all are no in selected chain!"),
            RangeInLocation::InSelectedChain(_hash_value, hash_values) => hash_values,
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
                found_common_header = Some(*result.last().unwrap());
                remote_start_id = *result.last().unwrap();
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

#[stest::test]
fn test_not_in_range_locate() -> anyhow::Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain_local =
        MockChain::new_with_genesis_for_test(net.clone(), genesis.clone(), 3)?;
    let mut mock_chain_remote = MockChain::new_with_genesis_for_test(net, genesis.clone(), 3)?;

    let count = 37;
    let _ = mock_chain_local.produce_and_apply_with_tips_for_times(count)?;
    let _ = mock_chain_remote.produce_and_apply_with_tips_for_times(count)?;

    let result = get_range_in_location(
        mock_chain_remote.head(),
        mock_chain_remote.head().get_storage(),
        mock_chain_local.head().current_header().id(),
        None,
    )?;

    assert_eq!(
        result,
        RangeInLocation::NotInSelectedChain,
        "expect not in selected chain"
    );

    anyhow::Ok(())
}

#[stest::test]
fn test_same_range_request() -> anyhow::Result<()> {
    let net = ChainNetwork::new_test();
    let genesis = Genesis::build(&net)?;
    let mut mock_chain_remote = MockChain::new_with_genesis_for_test(net, genesis.clone(), 3)?;

    let count = 8;
    let _ = mock_chain_remote.produce_and_apply_with_tips_for_times(count)?;

    match get_range_in_location(
        mock_chain_remote.head(),
        mock_chain_remote.head().get_storage(),
        mock_chain_remote.head().current_header().id(),
        Some(mock_chain_remote.head().current_header().id()),
    )? {
        RangeInLocation::NotInSelectedChain => bail!("expect in selected chain"),
        RangeInLocation::InSelectedChain(hash_value, hash_values) => {
            assert_eq!(hash_value, mock_chain_remote.head().current_header().id());
            assert_eq!(hash_values.len(), 0);
        }
    }

    let block_header = mock_chain_remote
        .head()
        .get_block_info_by_number(3)?
        .ok_or_else(|| format_err!("block info not found"))?;

    match get_range_in_location(
        mock_chain_remote.head(),
        mock_chain_remote.head().get_storage(),
        *block_header.block_id(),
        Some(*block_header.block_id()),
    )? {
        RangeInLocation::NotInSelectedChain => bail!("expect in selected chain"),
        RangeInLocation::InSelectedChain(hash_value, hash_values) => {
            assert_eq!(hash_value, block_header.block_id().clone());
            assert_eq!(hash_values.len(), 0);
        }
    }

    anyhow::Ok(())
}
