use crate::block_connector::{
    create_writeable_block_chain, gen_blocks, new_block, WriteBlockChainService,
};
use anyhow::Result;
use chain::BlockChain;
use config::NodeConfig;
use consensus::Consensus;
use crypto::HashValue;
use logger::prelude::*;
use starcoin_account_api::AccountInfo;
use starcoin_storage::Store;
use starcoin_txpool_mock_service::MockTxPoolService;
use starcoin_types::block::BlockHeader;
use starcoin_types::{block::Block, U256};
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use traits::WriteableChainService;
use traits::{ChainReader, ChainWriter};

async fn new_block_and_master() -> (Block, BlockChain) {
    let times = 5;
    let (mut writeable_block_chain_service, node_config, storage) =
        create_writeable_block_chain().await;
    gen_blocks(
        &node_config.net().consensus(),
        times,
        &mut writeable_block_chain_service,
    );
    let new_block = new_block(
        None,
        &node_config.net().consensus(),
        &mut writeable_block_chain_service,
    );
    let head_id = writeable_block_chain_service
        .get_master()
        .current_header()
        .id();
    let master = BlockChain::new(node_config.net().consensus(), head_id, storage).unwrap();
    (new_block, master)
}

async fn uncle_block_and_writeable_block_chain() -> (
    BlockHeader,
    WriteBlockChainService<MockTxPoolService>,
    Arc<NodeConfig>,
    Arc<dyn Store>,
) {
    // 1. chain
    let times = 5;
    let (mut writeable_block_chain_service, node_config, storage) =
        create_writeable_block_chain().await;
    gen_blocks(
        &node_config.net().consensus(),
        times,
        &mut writeable_block_chain_service,
    );

    // 2. new branch and uncle block
    let miner_account = AccountInfo::random();
    let tmp_head = writeable_block_chain_service
        .get_master()
        .get_header_by_number(times - 2)
        .unwrap()
        .unwrap()
        .id();
    let new_branch =
        BlockChain::new(node_config.net().consensus(), tmp_head, storage.clone()).unwrap();
    let (block_template, _) = new_branch
        .create_block_template(
            *miner_account.address(),
            Some(miner_account.public_key.clone()),
            None,
            Vec::new(),
            vec![],
            None,
        )
        .unwrap();
    let new_block = node_config
        .net()
        .consensus()
        .create_block(&new_branch, block_template)
        .unwrap();
    let uncle_block_header = new_block.header().clone();
    (
        uncle_block_header,
        writeable_block_chain_service,
        node_config,
        storage,
    )
}

fn apply_with_illegal_uncle(
    consensus_strategy: ConsensusStrategy,
    uncles: Vec<BlockHeader>,
    writeable_block_chain_service: &mut WriteBlockChainService<MockTxPoolService>,
    storage: Arc<dyn Store>,
) -> Result<Block> {
    let miner_account = AccountInfo::random();
    let (block_template, _) = writeable_block_chain_service
        .get_master()
        .create_block_template(
            *miner_account.address(),
            Some(miner_account.public_key.clone()),
            None,
            Vec::new(),
            uncles,
            None,
        )?;
    let new_block = consensus_strategy
        .create_block(writeable_block_chain_service.get_master(), block_template)?;

    let head_id = writeable_block_chain_service
        .get_master()
        .current_header()
        .id();
    let mut master = BlockChain::new(consensus_strategy, head_id, storage)?;
    master.apply(new_block.clone())?;
    Ok(new_block)
}

#[stest::test]
async fn test_verify_gas_limit_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.gas_used = u64::MAX;
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_body_hash_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.body_hash = HashValue::random();
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_parent_id_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.parent_hash = HashValue::random();
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_parent_not_exist() {
    // TODO
}

#[stest::test]
async fn test_verify_timestamp_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.timestamp = master.current_header().timestamp();
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_future_timestamp_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 1000;
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_consensus_failed() {
    // TODO: fix me, change test consensus to halley consensus.
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.difficulty = U256::from(1024u64);
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_ok());
}

#[stest::test]
async fn test_verify_switch_epoch_uncle_count_failed() {
    // TODO
}

#[stest::test]
async fn test_verify_can_not_be_uncle_is_member_failed() {
    let times = 5;
    let (mut writeable_block_chain_service, node_config, storage) =
        create_writeable_block_chain().await;
    gen_blocks(
        &node_config.net().consensus(),
        times,
        &mut writeable_block_chain_service,
    );

    let uncle_header = writeable_block_chain_service
        .get_master()
        .get_header_by_number(times - 2)
        .unwrap()
        .unwrap();
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    let apply_failed = apply_with_illegal_uncle(
        node_config.net().consensus(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_can_not_be_uncle_check_ancestor_failed() {
    // 1. chain
    let times = 7;
    let (mut writeable_block_chain_service, node_config, storage) =
        create_writeable_block_chain().await;
    gen_blocks(
        &node_config.net().consensus(),
        times,
        &mut writeable_block_chain_service,
    );

    // 2. new branch
    let miner_account = AccountInfo::random();
    let tmp_head = writeable_block_chain_service
        .get_master()
        .get_header_by_number(times - 3)
        .unwrap()
        .unwrap()
        .id();
    let mut new_branch =
        BlockChain::new(node_config.net().consensus(), tmp_head, storage.clone()).unwrap();

    for _i in 0..2 {
        let (block_template, _) = new_branch
            .create_block_template(
                *miner_account.address(),
                Some(miner_account.public_key.clone()),
                None,
                Vec::new(),
                vec![],
                None,
            )
            .unwrap();
        let new_block = node_config
            .net()
            .consensus()
            .create_block(&new_branch, block_template)
            .unwrap();
        new_branch.apply(new_block).unwrap();
    }

    // 3. new block
    let uncle_header = new_branch.current_header();
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    let apply_failed = apply_with_illegal_uncle(
        node_config.net().consensus(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_illegal_uncle_timestamp_failed() {
    let (mut uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain().await;
    uncle_header.timestamp = writeable_block_chain_service
        .get_master()
        .get_header(uncle_header.parent_hash)
        .unwrap()
        .unwrap()
        .timestamp();
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    let apply_failed = apply_with_illegal_uncle(
        node_config.net().consensus(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_illegal_uncle_future_timestamp_failed() {
    let (mut uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain().await;
    uncle_header.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 1000;
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    let apply_failed = apply_with_illegal_uncle(
        node_config.net().consensus(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_illegal_uncle_consensus_failed() {
    // TODO
}

#[stest::test]
async fn test_verify_state_root_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.state_root = HashValue::random();
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_block_used_gas_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.gas_used = 1;
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_txn_count_failed() {
    // TODO: fix me
    let (mut new_block, mut master) = new_block_and_master().await;
    let mut txns = Vec::new();
    txns.push(SignedUserTransaction::mock());
    let mut body = new_block.body.clone();
    body.transactions = txns;
    new_block.body = body;
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_accumulator_root_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.accumulator_root = HashValue::random();
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_block_accumulator_root_succ() {
    let (new_block, mut master) = new_block_and_master().await;
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_ok());
}

#[stest::test]
async fn test_verify_block_accumulator_root_failed() {
    let (mut new_block, mut master) = new_block_and_master().await;
    new_block.header.parent_block_accumulator_root = HashValue::random();
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

async fn test_verify_block_number_failed(flag: bool) {
    let (mut new_block, mut master) = new_block_and_master().await;
    if flag {
        new_block.header.number -= 1;
    } else {
        new_block.header.number += 1;
    }
    let apply_failed = master.apply(new_block);
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_block_illegal_number_failed() {
    test_verify_block_number_failed(false).await;
    test_verify_block_number_failed(true).await;
}

async fn test_verify_uncles_count(succ: bool) -> Result<Block> {
    let (uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain().await;
    let mut uncles = Vec::new();
    let times = if succ { 2 } else { 3 };
    for _i in 0..times {
        let mut tmp = uncle_header.clone();
        tmp.state_root = HashValue::random();
        uncles.push(tmp);
    }
    apply_with_illegal_uncle(
        node_config.net().consensus(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    )
}

#[stest::test]
async fn test_verify_uncles_count_failed() {
    assert!(test_verify_uncles_count(true).await.is_ok());
    let apply_failed = test_verify_uncles_count(false).await;
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_uncles_number_failed() {
    let (mut uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain().await;
    uncle_header.number = writeable_block_chain_service
        .get_master()
        .current_header()
        .number()
        + 1;
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    let apply_failed = apply_with_illegal_uncle(
        node_config.net().consensus(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}

#[stest::test]
async fn test_verify_uncles_epoch_number_failed() {
    // TODO
}

#[stest::test]
async fn test_verify_uncles_uncle_exist_failed() {
    let (uncle_header, mut writeable_block_chain_service, node_config, storage) =
        uncle_block_and_writeable_block_chain().await;
    let mut uncles = Vec::new();
    uncles.push(uncle_header);
    info!(
        "number 1 : {}",
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number()
    );

    let miner_account = AccountInfo::random();
    let (block_template, _) = writeable_block_chain_service
        .get_master()
        .create_block_template(
            *miner_account.address(),
            Some(miner_account.public_key.clone()),
            None,
            Vec::new(),
            uncles.clone(),
            None,
        )
        .unwrap();
    let new_block = node_config
        .net()
        .consensus()
        .create_block(writeable_block_chain_service.get_master(), block_template)
        .unwrap();
    writeable_block_chain_service
        .try_connect(new_block)
        .unwrap();

    info!(
        "number 2 : {}",
        writeable_block_chain_service
            .get_master()
            .current_header()
            .number()
    );
    let apply_failed = apply_with_illegal_uncle(
        node_config.net().consensus(),
        uncles,
        &mut writeable_block_chain_service,
        storage,
    );
    assert!(apply_failed.is_err());
    if let Err(apply_err) = apply_failed {
        error!("apply failed : {:?}", apply_err);
    }
}
