use crate::random_txn;
use actix::clock::{delay_for, Duration};
use actix::{Addr, Arbiter, System};
use anyhow::{format_err, Result};
use criterion::{BatchSize, Bencher};
use crypto::HashValue;
use libp2p::multiaddr::Multiaddr;
use logger::prelude::*;
use network_rpc::gen_client::{get_rpc_info, NetworkRpcClient};
use starcoin_account_api::AccountInfo;
use starcoin_bus::BusActor;
use starcoin_chain::{BlockChain, ChainActor, ChainActorRef};
use starcoin_config::{get_random_available_port, NodeConfig};
use starcoin_consensus::Consensus;
use starcoin_genesis::Genesis;
use starcoin_network::{NetworkActor, NetworkAsyncService};
use starcoin_network_api::NetworkService;
use starcoin_state_service::ChainStateActor;
use starcoin_sync::helper::{get_body_by_hash, get_headers_msg_for_common, get_headers_with_peer};
use starcoin_sync::Downloader;
use starcoin_txpool::{TxPool, TxPoolService};
use std::sync::Arc;
use storage::Storage;
use traits::ChainAsyncService;
use types::peer_info::{PeerId, PeerInfo, RpcInfo};

/// Benchmarking support for sync.
pub struct SyncBencher;

impl SyncBencher {
    pub fn sync_block(&self, num: u64) {
        let mut system = System::new("sync-bench");
        system.block_on(async move {
            let (_bus_1, addr_1, network_1, chain_1, _tx_pool, _storage_1) =
                create_node(Some(num), None).await.unwrap();
            let (_, _, network_2, chain_2, _, _) =
                create_node(None, Some((addr_1, network_1))).await.unwrap();
            let chain_2_clone = chain_2.clone();
            let downloader = Arc::new(Downloader::new(chain_2_clone));
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
                    delay_for(Duration::from_millis(1000)).await;
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

async fn create_node(
    num: Option<u64>,
    seed: Option<(Multiaddr, NetworkAsyncService)>,
) -> Result<(
    Addr<BusActor>,
    Multiaddr,
    NetworkAsyncService,
    ChainActorRef,
    TxPoolService,
    Arc<Storage>,
)> {
    let bus = BusActor::launch();

    // node config
    let mut config = NodeConfig::random_for_test();
    config.sync.full_sync_mode();
    let my_addr: Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", get_random_available_port())
        .parse()
        .unwrap();
    config.network.listen = my_addr.clone();
    if let Some((seed_listen, seed_net)) = seed {
        let seed_id = seed_net.identify().to_base58();
        let seed_addr: Multiaddr = format!("{}/p2p/{}", &seed_listen, seed_id).parse().unwrap();
        config.network.seeds = vec![seed_addr];
    }
    let node_config = Arc::new(config);

    let (storage, startup_info, genesis_hash) =
        Genesis::init_storage(node_config.as_ref()).expect("init storage by genesis fail.");

    let txpool = {
        let best_block_id = *startup_info.get_master();
        TxPool::start(
            node_config.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
    };

    let txpool_service = txpool.get_service();

    // network
    let key_pair = node_config.clone().network.network_keypair();
    let addr = PeerId::from_ed25519_public_key(key_pair.public_key.clone());
    let mut rpc_proto_info = Vec::new();
    let chain_rpc_proto_info = get_rpc_info();
    rpc_proto_info.push((
        chain_rpc_proto_info.0.into(),
        RpcInfo::new(chain_rpc_proto_info.1),
    ));
    let node_config_clone = node_config.clone();
    let bus_clone = bus.clone();
    let addr_clone = addr.clone();
    let (network, rpc_rx) = NetworkActor::launch(
        node_config_clone,
        bus_clone,
        genesis_hash,
        PeerInfo::new_for_test(addr_clone, rpc_proto_info),
    );

    // chain
    let txpool_service_clone = txpool_service.clone();
    let node_config_clone = node_config.clone();
    let genesis_startup_info_clone = startup_info.clone();
    let storage_clone = storage.clone();
    let bus_clone = bus.clone();
    let chain = Arbiter::new()
        .exec(move || -> ChainActorRef {
            ChainActor::launch(
                node_config_clone,
                genesis_startup_info_clone,
                storage_clone,
                bus_clone,
                txpool_service_clone,
                None,
            )
            .unwrap()
        })
        .await?;
    let state_service = ChainStateActor::launch(bus.clone(), storage.clone(), None).unwrap();
    // network rpc server
    let _ = network_rpc::start_network_rpc_server(
        rpc_rx,
        chain.clone(),
        storage.clone(),
        state_service,
        txpool_service.clone(),
    )?;

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
    Ok((bus, my_addr, network, chain, txpool_service, storage))
}
