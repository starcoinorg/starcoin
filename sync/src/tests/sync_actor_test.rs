use crate::download::DownloadActor;
use crate::message::{ProcessMessage, SyncMessage};
use crate::process::ProcessActor;
use crate::sync::SyncActor;
use actix::Addr;
use atomic_refcell::AtomicRefCell;
use chain::{
    gen_mem_chain_for_test, mem_chain::MemChainActor, message::ChainRequest, ChainActorRef,
};
use crypto::hash::CryptoHash;
use std::sync::Arc;
use tokio::time::{delay_for, Duration};
use traits::Chain;
use types::{
    block::{Block, BlockHeader},
    peer_info::PeerInfo,
};

fn genesis_block_for_test() -> Block {
    Block::new_nil_block_for_test(BlockHeader::genesis_block_header_for_test())
}

fn gen_mem_chain_actor(times: u64) -> (PeerInfo, ChainActorRef<MemChainActor>) {
    let peer = PeerInfo::random();
    let genesis_block = genesis_block_for_test();
    println!("genesis_block: {:?}", genesis_block.crypto_hash());
    let mut first_chain_actor_address = MemChainActor::launch(genesis_block.clone()).unwrap();
    if times > 0 {
        first_chain_actor_address.do_send(ChainRequest::CreateBlock(times));
    }
    let chain = ChainActorRef {
        address: first_chain_actor_address,
    };
    (peer, chain)
}

pub fn gen_mem_chain_actors(
    times: u64,
) -> (
    PeerInfo,
    ChainActorRef<MemChainActor>,
    PeerInfo,
    ChainActorRef<MemChainActor>,
) {
    let (first_peer, first_chain) = gen_mem_chain_actor(times);
    let (second_peer, second_chain) = gen_mem_chain_actor(0);
    (first_peer, first_chain, second_peer, second_chain)
}

#[actix_rt::test]
async fn test_process_actor_new_peer() {
    let new_peer = PeerInfo::random();
    let (my_peer, chain) = gen_mem_chain_actor(5);

    let mut process_actor = ProcessActor::launch(Arc::new(my_peer), chain).unwrap();
    process_actor
        .send(ProcessMessage::NewPeerMsg(None, new_peer))
        .await;
    delay_for(Duration::from_millis(50)).await;
}

#[actix_rt::test]
async fn test_actors() {
    // let first_peer = PeerInfo::random();
    // let second_peer = PeerInfo::random();
    // let first_p = Arc::new(first_peer.clone());
    // let second_p = Arc::new(second_peer.clone());
    // let genesis_block = genesis_block_for_test();
    // let mut first_chain_actor_address = MemChainActor::launch(genesis_block.clone()).unwrap();
    // first_chain_actor_address.do_send(ChainRequest::CreateBlock(5));
    // let mut second_chain_actor_address = MemChainActor::launch(genesis_block).unwrap();
    // let first_chain = ChainActorRef { address: first_chain_actor_address };
    // let second_chain = ChainActorRef { address: second_chain_actor_address };

    let (first_peer, first_chain, second_peer, second_chain) = gen_mem_chain_actors(5);
    let first_p = Arc::new(first_peer.clone());
    let second_p = Arc::new(second_peer.clone());

    let first_p_actor = ProcessActor::launch(Arc::clone(&first_p), first_chain.clone()).unwrap();
    let first_d_actor =
        DownloadActor::launch(first_p, first_chain).expect("launch DownloadActor failed.");
    let second_p_actor = ProcessActor::launch(Arc::clone(&second_p), second_chain.clone())
        .expect("launch ProcessActor failed.");
    let second_d_actor =
        DownloadActor::launch(second_p, second_chain).expect("launch DownloadActor failed.");

    //connect
    first_p_actor.do_send(ProcessMessage::NewPeerMsg(
        Some(second_d_actor),
        second_peer,
    ));
    second_p_actor.do_send(ProcessMessage::NewPeerMsg(Some(first_d_actor), first_peer));

    delay_for(Duration::from_millis(50)).await;
}

#[actix_rt::test]
async fn test_sync_actors() {
    // let first_peer = PeerInfo::random();
    // let second_peer = PeerInfo::random();
    // let first_p = Arc::new(first_peer.clone());
    // let second_p = Arc::new(second_peer.clone());
    // let genesis_block = genesis_block_for_test();
    // let mut first_chain_actor_address = MemChainActor::launch(genesis_block.clone()).unwrap();
    // first_chain_actor_address.do_send(ChainRequest::CreateBlock(5));
    // let mut second_chain_actor_address = MemChainActor::launch(genesis_block).unwrap();
    // let first_chain = ChainActorRef { address: first_chain_actor_address };
    // let second_chain = ChainActorRef { address: second_chain_actor_address };

    let (first_peer, first_chain, second_peer, second_chain) = gen_mem_chain_actors(5);
    let first_p = Arc::new(first_peer.clone());
    let second_p = Arc::new(second_peer.clone());

    let first_p_actor = ProcessActor::launch(Arc::clone(&first_p), first_chain.clone())
        .expect("launch ProcessActor failed.");
    let first_d_actor =
        DownloadActor::launch(first_p, first_chain.clone()).expect("launch DownloadActor failed.");
    let second_p_actor = ProcessActor::launch(Arc::clone(&second_p), second_chain.clone())
        .expect("launch ProcessActor failed.");
    let second_d_actor = DownloadActor::launch(second_p, second_chain.clone())
        .expect("launch DownloadActor failed.");

    let first_sync_actor = SyncActor::launch(first_p_actor, first_d_actor.clone()).unwrap();
    let second_sync_actor = SyncActor::launch(second_p_actor, second_d_actor.clone()).unwrap();

    //connect
    first_sync_actor.do_send(SyncMessage::ProcessMessage(ProcessMessage::NewPeerMsg(
        Some(second_d_actor),
        second_peer,
    )));
    second_sync_actor.do_send(SyncMessage::ProcessMessage(ProcessMessage::NewPeerMsg(
        Some(first_d_actor),
        first_peer,
    )));

    delay_for(Duration::from_millis(50)).await;

    let block_1 = first_chain.head_block().await.unwrap();
    let block_2 = second_chain.head_block().await.unwrap();
    println!(
        "{}:{}",
        block_1.header().number(),
        block_2.header().number()
    );
    assert_eq!(block_1.crypto_hash(), block_2.crypto_hash());
}
