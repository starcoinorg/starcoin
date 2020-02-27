use super::gen_chain_actors;
use crate::download::Downloader;
use crate::message::{BatchBodyMsg, BatchHeaderMsg, HashWithNumber};
use crate::process::Processor;
use crate::sync::SyncFlow;
use atomic_refcell::AtomicRefCell;
use chain::{gen_mem_chain_for_test, mem_chain::MemChain};
use crypto::{hash::CryptoHash, HashValue};
use std::sync::Arc;
use types::peer_info::PeerInfo;

fn gen_chains(times: u64) -> (PeerInfo, MemChain, PeerInfo, MemChain) {
    let first_peer = PeerInfo::random();
    let first_chain = gen_mem_chain_for_test(times);
    let second_peer = PeerInfo::random();
    let second_chain = gen_mem_chain_for_test(0);
    (first_peer, first_chain, second_peer, second_chain)
}

fn gen_syncs(times: u64) -> (SyncFlow, SyncFlow) {
    let (first_peer, first_chain, second_peer, second_chain) = gen_chain_actors(times);

    let first = SyncFlow::new(first_peer, first_chain);
    let second = SyncFlow::new(second_peer, second_chain);
    (first, second)
}

async fn find_ancestor(first: &mut SyncFlow, second: &mut SyncFlow) -> HashWithNumber {
    let latest_state_msg_1 = Processor::send_latest_state_msg(first.processor.clone()).await;
    let latest_state_msg_2 = Processor::send_latest_state_msg(second.processor.clone()).await;

    Downloader::handle_latest_state_msg(
        first.downloader.clone(),
        second.peer_info.clone(),
        latest_state_msg_2,
    )
    .await;
    Downloader::handle_latest_state_msg(
        second.downloader.clone(),
        first.peer_info.clone(),
        latest_state_msg_1,
    )
    .await;

    let get_hash_by_number_msg = Downloader::send_get_hash_by_number_msg(second.downloader.clone())
        .await
        .expect("get_hash_by_number_msg is none.");

    let batch_hash_by_number_msg =
        Processor::handle_get_hash_by_number_msg(first.processor.clone(), get_hash_by_number_msg)
            .await;

    Downloader::find_ancestor(
        second.downloader.clone(),
        first.peer_info.clone(),
        batch_hash_by_number_msg,
    )
    .await
    .expect("ancestor is none.")
}

async fn sync_header(first: &mut SyncFlow, second: &mut SyncFlow) -> BatchHeaderMsg {
    find_ancestor(first, second).await;
    let get_header_by_hash_msg = Downloader::send_get_header_by_hash_msg(second.downloader.clone())
        .await
        .expect("header is none.");
    Processor::handle_get_header_by_hash_msg(first.processor.clone(), get_header_by_hash_msg).await
}

async fn sync_body(first: &mut SyncFlow, second: &mut SyncFlow) -> BatchBodyMsg {
    let batch_header_msg = sync_header(first, second).await;
    Downloader::handle_batch_header_msg(
        second.downloader.clone(),
        first.peer_info.clone(),
        batch_header_msg.clone(),
    )
    .await;
    let get_body_by_hash_msg = Downloader::send_get_body_by_hash_msg(second.downloader.clone())
        .await
        .expect("body is none.");
    Processor::handle_get_body_by_hash_msg(first.processor.clone(), get_body_by_hash_msg).await
}

#[actix_rt::test]
async fn test_find_ancestor() {
    let (mut first, mut second) = gen_syncs(5);
    let ancestor = find_ancestor(&mut first, &mut second).await;
    assert_eq!(ancestor.number, 0);
}

#[actix_rt::test]
async fn test_sync_header() {
    let (mut first, mut second) = gen_syncs(5);
    let batch_header_msg = sync_header(&mut first, &mut second).await;
    println!("{:?}", batch_header_msg.headers.len());
    assert!(!batch_header_msg.headers.is_empty());
}

#[actix_rt::test]
async fn test_sync_body() {
    let (mut first, mut second) = gen_syncs(5);
    let batch_body_msg = sync_body(&mut first, &mut second).await;
    assert!(!batch_body_msg.bodies.is_empty());
}

#[actix_rt::test]
async fn test_do_block() {
    let (mut first, mut second) = gen_syncs(5);
    let batch_header_msg = sync_header(&mut first, &mut second).await;
    Downloader::handle_batch_header_msg(
        second.downloader.clone(),
        first.peer_info.clone(),
        batch_header_msg.clone(),
    )
    .await;
    let get_body_by_hash_msg = Downloader::send_get_body_by_hash_msg(second.downloader.clone())
        .await
        .expect("body is none.");
    let batch_body_msg =
        Processor::handle_get_body_by_hash_msg(first.processor.clone(), get_body_by_hash_msg).await;
    Downloader::do_blocks(
        second.downloader.clone(),
        batch_header_msg.headers,
        batch_body_msg.bodies,
    )
    .await;

    let block_1 = Processor::head_block(first.processor.clone()).await;
    let block_2 = Processor::head_block(second.processor.clone()).await;
    println!(
        "{}:{}",
        block_1.header().number(),
        block_2.header().number()
    );
    assert_eq!(block_1.crypto_hash(), block_2.crypto_hash());
}
