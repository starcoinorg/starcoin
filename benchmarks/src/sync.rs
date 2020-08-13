use crate::random_txn;
use actix::System;
use anyhow::{format_err, Result};
use criterion::{BatchSize, Bencher};
use crypto::HashValue;
use futures_timer::Delay;
use logger::prelude::*;
use network_rpc::gen_client::NetworkRpcClient;
use starcoin_account_api::AccountInfo;
use starcoin_chain::BlockChain;
use starcoin_config::{NodeConfig, SyncMode};
use starcoin_consensus::Consensus;
use starcoin_network::NetworkAsyncService;
use starcoin_network_api::NetworkService;
use starcoin_node::node::NodeStartHandle;
use starcoin_sync::helper::{get_body_by_hash, get_headers_msg_for_common, get_headers_with_peer};
use starcoin_sync::Downloader;
use std::sync::Arc;
use std::time::Duration;
use traits::ChainAsyncService;

/// Benchmarking support for sync.
pub struct SyncBencher;

impl SyncBencher {
    pub fn sync_block(&self, num: u64) {
        let mut system = System::new("sync-bench");
        system.block_on(async move {
            let mut config_1 = NodeConfig::random_for_test();
            config_1.sync.set_mode(SyncMode::FULL);
            let config_1 = Arc::new(config_1);

            let node_handle_1 = create_node(Some(num), config_1.clone()).await.unwrap();
            let chain_1 = node_handle_1.chain_actor;

            let mut config_2 = NodeConfig::random_for_test();
            config_2.sync.set_mode(SyncMode::FULL);
            config_2.network.seeds = vec![config_1.network.self_address().unwrap()];
            let config_2 = Arc::new(config_2);

            let node_handle_2 = create_node(None, config_2).await.unwrap();
            let chain_2 = node_handle_2.chain_actor;
            let network_2 = node_handle_2.network;

            let downloader = Arc::new(Downloader::new(chain_2.clone()));
            let rpc_client = NetworkRpcClient::new(network_2.clone());

            for i in 0..3 {
                SyncBencher::sync_block_inner(
                    downloader.clone(),
                    rpc_client.clone(),
                    network_2.clone(),
                )
                .await
                .unwrap();
                let first = chain_1.clone().master_head().await.unwrap();
                let second = chain_2.clone().master_head().await.unwrap();
                if first.get_head() != second.get_head() {
                    info!("full sync failed: {}", i);
                    Delay::new(Duration::from_millis(1000)).await;
                } else {
                    break;
                }
            }
            let first = chain_1.clone().master_head().await.unwrap();
            let second = chain_2.clone().master_head().await.unwrap();
            assert_eq!(first.get_head(), second.get_head());
            info!("full sync test ok.");
        });
    }

    pub fn bench_full_sync(&self, b: &mut Bencher, num: u64) {
        b.iter_batched(
            || (self, num),
            |(bench, n)| bench.sync_block(n),
            BatchSize::LargeInput,
        )
    }

    async fn sync_block_inner(
        downloader: Arc<Downloader>,
        rpc_client: NetworkRpcClient<NetworkAsyncService>,
        network: NetworkAsyncService,
    ) -> Result<()> {
        if let Some(best_peer) = network.best_peer().await? {
            if let Some(header) = downloader.get_chain_reader().master_head_header().await? {
                let begin_number = header.number();
                let end_number = best_peer.get_block_number();

                let total_difficulty = downloader
                    .get_chain_reader()
                    .get_block_info_by_hash(&header.id())
                    .await?
                    .ok_or_else(|| format_err!("Master head block info is none."))?
                    .total_difficulty;

                if let Some(ancestor_header) = downloader
                    .find_ancestor_header(
                        best_peer.get_peer_id(),
                        &rpc_client,
                        network.clone(),
                        begin_number,
                        total_difficulty,
                        true,
                    )
                    .await?
                {
                    let mut latest_block_id = ancestor_header.id();
                    let mut latest_number = ancestor_header.number();
                    loop {
                        if end_number <= latest_number {
                            break;
                        }
                        let get_headers_req = get_headers_msg_for_common(latest_block_id);
                        let headers = get_headers_with_peer(
                            &rpc_client,
                            best_peer.get_peer_id(),
                            get_headers_req,
                            latest_number,
                        )
                        .await?;
                        let latest_header = headers.last().expect("headers is empty.");
                        latest_block_id = latest_header.id();
                        latest_number = latest_header.number();
                        let hashes: Vec<HashValue> =
                            headers.iter().map(|header| header.id()).collect();
                        //TODO: get_body_by_hash select a best peer again.which maybe different to best peer selected before.
                        let (bodies, _) =
                            get_body_by_hash(&rpc_client, &network, hashes.clone()).await?;
                        info!(
                            "sync block number : {:?} from peer {:?}",
                            latest_number,
                            best_peer.get_peer_id()
                        );
                        downloader
                            .do_blocks(headers, bodies, best_peer.get_peer_id())
                            .await;
                    }
                }
            }
        }

        Ok(())
    }
}

async fn create_node(num: Option<u64>, node_config: Arc<NodeConfig>) -> Result<NodeStartHandle> {
    let node_handle = starcoin_node::node::start(node_config.clone(), None).await?;
    let chain = node_handle.chain_actor.clone();
    let storage = node_handle.storage.clone();
    if let Some(n) = num {
        let miner_account = AccountInfo::random();
        for i in 0..n {
            info!(
                "create block: {:?} : {:?}",
                &node_config.network.self_peer_id, i
            );
            let startup_info = chain.clone().master_startup_info().await?;

            let block_chain = BlockChain::new(
                node_config.net(),
                startup_info.master,
                storage.clone(),
                None,
            )
            .unwrap();

            let mut txn_vec = Vec::new();
            txn_vec.push(random_txn(i + 1));
            let block_template = chain
                .clone()
                .create_block_template(
                    *miner_account.address(),
                    Some(miner_account.get_auth_key().prefix().to_vec()),
                    None,
                    txn_vec,
                )
                .await
                .unwrap();
            let block = node_config
                .net()
                .consensus()
                .create_block(&block_chain, block_template)
                .unwrap();
            chain.clone().try_connect(block).await.unwrap();
        }
    }
    Ok(node_handle)
}
