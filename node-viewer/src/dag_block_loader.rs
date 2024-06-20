use std::path::Path;
use std::sync::Arc;

use anyhow::format_err;

use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_dag::{blockdag::DEFAULT_GHOSTDAG_K, consensusdb::prelude::FlexiDagStorageConfig};
use starcoin_genesis::Genesis;
use starcoin_storage::{
    cache_storage::CacheStorage, db_storage::DBStorage, storage::StorageInstance, Storage,
    StorageVersion,
};
use starcoin_types::{block::Block};
use starcoin_types::consensus_header::ConsensusHeader;

use crate::dagre_dag_viewer::DagNode;

const DEFAULT_BLOCK_GAP: u64 = 20;

pub fn load_blocks_from_db(
    network: BuiltinNetworkID,
    start: Option<u64>,
    end: Option<u64>,
    root_path: &Path,
) -> anyhow::Result<Vec<DagNode>> {
    let net = ChainNetwork::new_builtin(network);
    let db_storage = DBStorage::open_with_cfs(
        root_path.join("starcoindb/db/starcoindb"),
        StorageVersion::current_version()
            .get_column_family_names()
            .to_vec(),
        true,
        Default::default(),
        None,
    )?;
    let dag_storage = starcoin_dag::consensusdb::prelude::FlexiDagStorage::create_from_path(
        root_path.join("dag/db/starcoindb"),
        FlexiDagStorageConfig::new(),
    )?;
    let dag = starcoin_dag::blockdag::BlockDAG::new(DEFAULT_GHOSTDAG_K, dag_storage);

    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        CacheStorage::new(None),
        db_storage,
    ))?);
    let (chain_info, _) =
        Genesis::init_and_check_storage(&net, storage.clone(), dag.clone(), root_path.as_ref())?;
    let chain = BlockChain::new(
        net.time_service(),
        chain_info.head().id(),
        storage,
        None,
        dag,
    )
    .expect("create block chain should success.");
    let cur_num = chain.status().head().number();
    let start_num = start.unwrap_or_else(|| {
        if cur_num > DEFAULT_BLOCK_GAP {
            cur_num - DEFAULT_BLOCK_GAP
        } else {
            1
        }
    });
    let end_num = end.unwrap_or_else(|| cur_num);
    if start_num > end_num {
        return Err(format_err!("start number should less than end number"));
    }

    let block_list = (start_num..=end_num)
        .into_iter()
        .map(|block_number| {
            let block = chain
                .get_block_by_number(block_number).expect("Get blockfailed")
                .ok_or_else(|| format_err!("{} get block error", block_number))
                .expect("Get block failed");
            block
        })
        .collect::<Vec<Block>>();

    let mut nodes = vec![];
    for block in block_list {
        let header = block.header();
        println!("header.parents() : {:?}", header.parents());
        let parents = header
            .parents_hash()
            .unwrap()
            .iter()
            .map(|h| h.to_hex_literal())
            .collect::<Vec<String>>();

        nodes.push(DagNode::new(
            header.id().to_hex_literal().as_str(),
            &parents,
            Some(header.number()),
            Some(header.timestamp()),
            Some(header.author().to_hex_literal()),
        ));
    }
    Ok(nodes)
}
