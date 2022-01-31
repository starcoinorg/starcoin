// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use sp_utils::stop_watch::start_watch;
use starcoin_chain::verifier::Verifier;
use starcoin_chain::verifier::{BasicVerifier, ConsensusVerifier, FullVerifier, NoneVerifier};
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_config::RocksdbConfig;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_genesis::Genesis;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::StorageInstance;
use starcoin_storage::{BlockStore, Storage};
use starcoin_types::startup_info::StartupInfo;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "replay")]
pub struct ReplayOpt {
    #[structopt(long, short = "n")]
    /// Chain Network to replay.
    pub net: Option<BuiltinNetworkID>,
    #[structopt(short = "f", long, parse(from_os_str))]
    /// Replay data dir.
    pub from: PathBuf,
    #[structopt(short = "t", long, parse(from_os_str))]
    /// Target dir.
    pub to: PathBuf,
    #[structopt(long)]
    /// replay from start to this block, include this block. default to all.
    pub end_block: Option<u64>,
    #[structopt(long, short = "c", default_value = "20000")]
    /// Number of block.
    pub block_num: u64,
    #[structopt(possible_values = &Verifier::variants(), case_insensitive = true)]
    /// Verify type:  Basic, Consensus, Full, None, eg.
    pub verifier: Verifier,
    #[structopt(long, short = "w")]
    /// Watch metrics logs.
    pub watch: bool,
}

// deprecated use starcoin_db_exporter replace
fn main() -> anyhow::Result<()> {
    let _logger = starcoin_logger::init();
    let opts: ReplayOpt = ReplayOpt::from_args();

    let network = match opts.net {
        Some(network) => network,
        None => BuiltinNetworkID::Proxima,
    };
    let net = ChainNetwork::new_builtin(network);

    let from_dir = opts.from;
    let end_block = opts.end_block;

    let to_dir = opts.to;
    // start watching
    if opts.watch {
        start_watch();
    }

    let db_storage = DBStorage::new(
        from_dir.join("starcoindb/db"),
        RocksdbConfig::default(),
        None,
    )
    .unwrap();

    let storage = Arc::new(
        Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::new(None),
            db_storage,
        ))
        .unwrap(),
    );
    let (chain_info, _) = Genesis::init_and_check_storage(&net, storage.clone(), from_dir.as_ref())
        .expect("init storage by genesis fail.");
    let chain = BlockChain::new(net.time_service(), chain_info.head().id(), storage, None)
        .expect("create block chain should success.");

    let storage2 = Arc::new(
        Storage::new(StorageInstance::new_cache_and_db_instance(
            CacheStorage::new(None),
            DBStorage::new(to_dir.join("starcoindb"), RocksdbConfig::default(), None).unwrap(),
        ))
        .unwrap(),
    );
    let (chain_info2, _) = Genesis::init_and_check_storage(&net, storage2.clone(), to_dir.as_ref())
        .expect("init storage by genesis fail.");

    let mut chain2 = BlockChain::new(
        net.time_service(),
        chain_info2.status().head().id(),
        storage2.clone(),
        None,
    )
    .expect("create block chain should success.");

    //read from first chain

    let time_begin = Instant::now();

    let end_block = end_block.unwrap_or_else(|| chain.current_header().number());

    {
        let b = chain2.get_block_by_number(end_block)?;
        if let Some(h) = b {
            if h.id() == chain.current_header().id() {
                println!("target chain already synced with source chain");
            } else {
                println!("target chain have different block with source chain at latest block: {}, target: {}, source: {}",
                             end_block,
                             h.id(),
                             chain.current_header().id(),
                    );
            }
            return Ok(());
        }
    }
    let start_block = chain2.current_header().number() + 1;
    let mut last_block_hash = None;
    for i in start_block..=end_block {
        if let Ok(Some(block)) = chain.get_block_by_number(i) {
            let start = Instant::now();
            let expected_state_root = block.header().state_root();
            let block_id = block.id();
            let block_height = block.header().number();
            match opts.verifier {
                Verifier::Basic => {
                    chain2.apply_with_verifier::<BasicVerifier>(block).unwrap();
                }
                Verifier::Consensus => {
                    chain2
                        .apply_with_verifier::<ConsensusVerifier>(block)
                        .unwrap();
                }
                Verifier::None => {
                    chain2.apply_with_verifier::<NoneVerifier>(block).unwrap();
                }
                Verifier::Full => {
                    chain2.apply_with_verifier::<FullVerifier>(block).unwrap();
                }
            };
            println!(
                "apply block {} at height: {}, time_used: {:?}, source state root: {}, target state root: {}",
                block_id,
                block_height,
                start.elapsed(),
                expected_state_root,
                chain2.chain_state_reader().state_root()
            );
            last_block_hash = Some(block_id);

            // save start up info every 100 blocks
            if i % 100 == 0 {
                if let Some(last_block_hash) = last_block_hash {
                    let startup_info = StartupInfo::new(last_block_hash);
                    storage2.save_startup_info(startup_info)?;
                }
            }
        } else {
            println!("read source block err, number : {:?}", i);
            break;
        }
    }

    if let Some(last_block_hash) = last_block_hash {
        let startup_info = StartupInfo::new(last_block_hash);
        storage2.save_startup_info(startup_info)?;
    }
    println!("apply use time: {:?}", time_begin.elapsed());
    Ok(())
}
