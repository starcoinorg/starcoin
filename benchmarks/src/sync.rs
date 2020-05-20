use crate::random_txn;
use actix::{Addr, System};
use anyhow::Result;
use criterion::{BatchSize, Bencher};
use libp2p::multiaddr::Multiaddr;
use starcoin_bus::BusActor;
use starcoin_chain::{to_block_chain_collection, BlockChain, ChainActor, ChainActorRef};
use starcoin_config::{get_available_port, NodeConfig};
use starcoin_consensus::dummy::DummyConsensus;
use starcoin_genesis::Genesis;
use starcoin_network::{NetworkActor, NetworkAsyncService, RawRpcRequestMessage};
use starcoin_network_api::NetworkService;
use starcoin_sync::Downloader;
use starcoin_sync::{
    helper::{get_block_by_hash, get_hash_by_number},
    ProcessActor,
};
use starcoin_sync_api::SyncMetadata;
use starcoin_txpool::{TxPool, TxPoolRef};
use starcoin_wallet_api::WalletAccount;
use std::sync::Arc;
use storage::cache_storage::CacheStorage;
use storage::storage::StorageInstance;
use storage::Storage;
use tokio::runtime::Handle;
use traits::{ChainAsyncService, Consensus};
use types::peer_info::{PeerId, PeerInfo};

/// Benchmarking support for sync.
pub struct SyncBencher;

impl SyncBencher {
    pub fn sync_block(&self, num: u64) {
        let mut system = System::new("sync-bench");
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle().clone();
        system.block_on(async move {
            let (bus_1, addr_1, network_1, chain_1, tx_1, storage_1, rpc_rx) =
                create_node(Some(num), None, handle.clone()).await.unwrap();
            let _processor = ProcessActor::launch(chain_1.clone(), tx_1, bus_1, storage_1, rpc_rx);

            let (_, _, network_2, chain_2, _, _, _) =
                create_node(None, Some((addr_1, network_1)), handle.clone())
                    .await
                    .unwrap();
            let downloader = Arc::new(Downloader::new(chain_2.clone()));
            SyncBencher::sync_block_inner(downloader, network_2)
                .await
                .unwrap();
            let first = chain_1.clone().master_head().await.unwrap();
            let second = chain_2.clone().master_head().await.unwrap();
            if first.get_head() != second.get_head() {
                println!("{:?}", first);
                println!("{:?}", second);
            }
            // assert_eq!(
            //     chain_1.master_head().await.unwrap().get_head(),
            //     chain_2.master_head().await.unwrap().get_head()
            // );
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
        downloader: Arc<Downloader<DummyConsensus>>,
        network: NetworkAsyncService,
    ) -> Result<()> {
        if let Some(best_peer) = network.best_peer().await? {
            if let Some(header) = downloader.get_chain_reader().master_head_header().await {
                let mut begin_number = header.number();

                if let Some(hash_number) = Downloader::find_ancestor(
                    downloader.clone(),
                    best_peer.get_peer_id(),
                    network.clone(),
                    begin_number,
                )
                .await?
                {
                    begin_number = hash_number.number + 1;
                    while let Some((get_hash_by_number_msg, end, next_number)) =
                        Downloader::<DummyConsensus>::get_hash_by_number_msg_forward(
                            network.clone(),
                            best_peer.get_peer_id(),
                            begin_number,
                        )
                        .await?
                    {
                        begin_number = next_number;
                        let batch_hash_by_number_msg = get_hash_by_number(
                            &network,
                            best_peer.get_peer_id(),
                            get_hash_by_number_msg,
                        )
                        .await?;

                        Downloader::put_hash_2_hash_pool(
                            downloader.clone(),
                            best_peer.get_peer_id(),
                            batch_hash_by_number_msg,
                        );

                        let hashs = Downloader::take_task_from_hash_pool(downloader.clone());
                        if !hashs.is_empty() {
                            let (headers, bodies, infos) =
                                get_block_by_hash(&network, best_peer.get_peer_id(), hashs).await?;
                            Downloader::do_blocks(
                                downloader.clone(),
                                headers.headers,
                                bodies.bodies,
                                infos.infos,
                            )
                            .await;
                        }

                        if end {
                            break;
                        }
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
    handle: Handle,
) -> Result<(
    Addr<BusActor>,
    Multiaddr,
    NetworkAsyncService,
    ChainActorRef<DummyConsensus>,
    TxPoolRef,
    Arc<Storage>,
    futures::channel::mpsc::UnboundedReceiver<RawRpcRequestMessage>,
)> {
    let bus = BusActor::launch();
    // storage
    let storage =
        Arc::new(Storage::new(StorageInstance::new_cache_instance(CacheStorage::new())).unwrap());
    // node config
    let mut config = NodeConfig::random_for_test();
    config.sync.full_sync_mode();
    let my_addr: Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", get_available_port())
        .parse()
        .unwrap();
    config.network.listen = my_addr.clone();
    if let Some((seed_listen, seed_net)) = seed {
        let seed_id = seed_net.identify().to_base58();
        let seed_addr: Multiaddr = format!("{}/p2p/{}", &seed_listen, seed_id).parse().unwrap();
        config.network.seeds = vec![seed_addr];
    }
    let node_config = Arc::new(config);

    // genesis
    let genesis = Genesis::build(node_config.net()).unwrap();
    let genesis_hash = genesis.block().header().id();
    let genesis_startup_info = genesis.execute(storage.clone()).unwrap();
    let txpool = {
        let best_block_id = *genesis_startup_info.get_master();
        TxPool::start(
            node_config.tx_pool.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
        .get_async_service()
    };

    // network
    let key_pair = node_config.clone().network.network_keypair();
    let addr = PeerId::from_ed25519_public_key(key_pair.public_key.clone());
    let (network, rpc_rx) = NetworkActor::launch(
        node_config.clone(),
        bus.clone(),
        handle.clone(),
        genesis_hash,
        PeerInfo::new_for_test(addr.clone()),
    );

    let sync_metadata_actor = SyncMetadata::new(node_config.clone(), bus.clone());
    let _ = sync_metadata_actor.block_sync_done();
    // chain
    let chain = ChainActor::launch(
        node_config.clone(),
        genesis_startup_info.clone(),
        storage.clone(),
        Some(network.clone()),
        bus.clone(),
        txpool.clone(),
        sync_metadata_actor.clone(),
    )
    .unwrap();

    if let Some(n) = num {
        let miner_account = WalletAccount::random();
        for i in 0..n {
            let startup_info = chain.clone().master_startup_info().await?;
            let collection = to_block_chain_collection(
                node_config.clone(),
                startup_info.clone(),
                storage.clone(),
            )?;

            let block_chain = BlockChain::<DummyConsensus, Storage>::new(
                node_config.clone(),
                collection.get_head(),
                storage.clone(),
                Arc::downgrade(&collection),
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
            let block =
                DummyConsensus::create_block(node_config.clone(), &block_chain, block_template)
                    .unwrap();
            chain.clone().try_connect(block).await.unwrap().unwrap();
        }
    }
    Ok((bus, my_addr, network, chain, txpool, storage, rpc_rx))
}
