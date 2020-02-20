use crate::proto::{BatchBodyMsg, BatchHeaderMsg, HashWithNumber};
use crate::sync::SyncFlow;
use atomic_refcell::AtomicRefCell;
use chain::{gen_mem_chain_for_test, mem_chain::MemChain};
use crypto::{hash::CryptoHash, HashValue};
use std::sync::Arc;
use types::peer_info::PeerInfo;

pub fn gen_chains(times: u64) -> (PeerInfo, MemChain, PeerInfo, MemChain) {
    let first_peer = PeerInfo::random();
    let first_chain = gen_mem_chain_for_test(times);
    let second_peer = PeerInfo::random();
    let second_chain = gen_mem_chain_for_test(0);
    (first_peer, first_chain, second_peer, second_chain)
}

fn gen_syncs(times: u64) -> (SyncFlow, SyncFlow) {
    let (first_peer, first_chain, second_peer, second_chain) = gen_chains(times);

    let first = SyncFlow::new(first_peer, Arc::new(AtomicRefCell::new(first_chain)));
    let second = SyncFlow::new(second_peer, Arc::new(AtomicRefCell::new(second_chain)));
    (first, second)
}

fn find_ancestor(first: &mut SyncFlow, second: &mut SyncFlow) -> HashWithNumber {
    let latest_state_msg_1 = first.processor.send_latest_state_msg();
    let latest_state_msg_2 = second.processor.send_latest_state_msg();

    first
        .downloader
        .handle_latest_state_msg(second.peer_info.clone(), latest_state_msg_2);
    second
        .downloader
        .handle_latest_state_msg(first.peer_info.clone(), latest_state_msg_1);

    let get_hash_by_number_msg = second
        .downloader
        .send_get_hash_by_number_msg()
        .expect("get_hash_by_number_msg is none.");

    let batch_hash_by_number_msg = first
        .processor
        .handle_get_hash_by_number_msg(get_hash_by_number_msg);

    second
        .downloader
        .find_ancestor(first.peer_info.clone(), batch_hash_by_number_msg)
        .expect("ancestor is none.")
}

fn sync_header(first: &mut SyncFlow, second: &mut SyncFlow) -> BatchHeaderMsg {
    find_ancestor(first, second);
    let get_header_by_hash_msg = second
        .downloader
        .send_get_header_by_hash_msg()
        .expect("header is none.");
    first
        .processor
        .handle_get_header_by_hash_msg(get_header_by_hash_msg)
}

fn sync_body(first: &mut SyncFlow, second: &mut SyncFlow) -> BatchBodyMsg {
    let batch_header_msg = sync_header(first, second);
    second
        .downloader
        .handle_batch_header_msg(first.peer_info.clone(), batch_header_msg.clone());
    let get_body_by_hash_msg = second
        .downloader
        .send_get_body_by_hash_msg()
        .expect("body is none.");
    first
        .processor
        .handle_get_body_by_hash_msg(get_body_by_hash_msg)
}

#[test]
fn test_find_ancestor() {
    let (mut first, mut second) = gen_syncs(5);
    let ancestor = find_ancestor(&mut first, &mut second);
    assert_eq!(ancestor.number, 0);
}

#[test]
fn test_sync_header() {
    let (mut first, mut second) = gen_syncs(5);
    let batch_header_msg = sync_header(&mut first, &mut second);
    println!("{:?}", batch_header_msg.headers.len());
    assert!(!batch_header_msg.headers.is_empty());
}

#[test]
fn test_sync_body() {
    let (mut first, mut second) = gen_syncs(5);
    let batch_body_msg = sync_body(&mut first, &mut second);
    assert!(!batch_body_msg.bodies.is_empty());
}

#[test]
fn test_do_block() {
    let (mut first, mut second) = gen_syncs(5);
    let batch_header_msg = sync_header(&mut first, &mut second);
    second
        .downloader
        .handle_batch_header_msg(first.peer_info.clone(), batch_header_msg.clone());
    let get_body_by_hash_msg = second
        .downloader
        .send_get_body_by_hash_msg()
        .expect("body is none.");
    let batch_body_msg = first
        .processor
        .handle_get_body_by_hash_msg(get_body_by_hash_msg);
    second
        .downloader
        .do_block(batch_header_msg.headers, batch_body_msg.bodies);

    let block_1 = first.processor.head_block();
    let block_2 = second.processor.head_block();
    println!(
        "{}:{}",
        block_1.header().number(),
        block_2.header().number()
    );
    assert_eq!(block_1.crypto_hash(), block_2.crypto_hash());
}
