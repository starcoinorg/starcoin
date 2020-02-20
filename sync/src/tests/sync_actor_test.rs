use super::gen_chains;
use crate::download::DownloadActor;
use crate::process::ProcessActor;
use crate::proto::{ProcessMessage, SyncMessage};
use crate::sync::SyncActor;
use actix::Addr;
use atomic_refcell::AtomicRefCell;
use chain::{gen_mem_chain_for_test, mem_chain::MemChain};
use std::sync::Arc;
use tokio::time::{delay_for, Duration};
use types::peer_info::PeerInfo;

#[actix_rt::test]
async fn test_process_actor_new_peer() {
    let my_peer = PeerInfo::random();
    let new_peer = PeerInfo::random();
    let chain = gen_mem_chain_for_test(5);
    let actor = ProcessActor::launch(Arc::new(my_peer), Arc::new(AtomicRefCell::new(chain)))
        .expect("launch ProcessActor failed.");
    actor.send(ProcessMessage::NewPeerMsg(None, new_peer));
    delay_for(Duration::from_millis(20)).await;
}

#[actix_rt::test]
async fn test_actors() {
    let (first_peer, first_mem_chain, second_peer, second_mem_chain) = gen_chains(5);
    let first_p = Arc::new(first_peer.clone());
    let second_p = Arc::new(second_peer.clone());
    let first_chain = Arc::new(AtomicRefCell::new(first_mem_chain));
    let second_chain = Arc::new(AtomicRefCell::new(second_mem_chain));
    let first_p_actor = ProcessActor::launch(Arc::clone(&first_p), Arc::clone(&first_chain))
        .expect("launch ProcessActor failed.");
    let first_d_actor =
        DownloadActor::launch(first_p, first_chain).expect("launch DownloadActor failed.");
    let second_p_actor = ProcessActor::launch(Arc::clone(&second_p), Arc::clone(&second_chain))
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
    let (first_peer, first_mem_chain, second_peer, second_mem_chain) = gen_chains(5);
    let first_p = Arc::new(first_peer.clone());
    let second_p = Arc::new(second_peer.clone());
    let first_chain = Arc::new(AtomicRefCell::new(first_mem_chain));
    let second_chain = Arc::new(AtomicRefCell::new(second_mem_chain));
    let first_p_actor = ProcessActor::launch(Arc::clone(&first_p), Arc::clone(&first_chain))
        .expect("launch ProcessActor failed.");
    let first_d_actor =
        DownloadActor::launch(first_p, first_chain).expect("launch DownloadActor failed.");
    let second_p_actor = ProcessActor::launch(Arc::clone(&second_p), Arc::clone(&second_chain))
        .expect("launch ProcessActor failed.");
    let second_d_actor =
        DownloadActor::launch(second_p, second_chain).expect("launch DownloadActor failed.");

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
}
