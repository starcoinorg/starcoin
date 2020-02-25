use super::random_block;
use crate::chain::load_genesis_block;
use crate::{AsyncChain, BlockChain, ChainActor, ChainActorRef, ChainAsyncService};
use actix::Addr;
use config::NodeConfig;
use crypto::hash::CryptoHash;
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, StarcoinStorage};

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

fn gen_block_chain_actor() -> ChainActorRef<BlockChainActor> {
    let repo = Arc::new(MemoryStorage::new());
    let storage = Arc::new(StarcoinStorage::new(repo).unwrap());
    let chain_actor = ChainActor::launch(Arc::new(NodeConfig::default()), storage.clone()).unwrap();
    ChainActorRef {
        address: chain_actor,
    };
}

#[actix_rt::test]
async fn test_block_chain_head() {
    let chain = gen_block_chain_actor();
    let genesis_block = load_genesis_block();
    let times = 5;
    let mut parent_hash = genesis_block.crypto_hash();
    for i in 0..times {
        let block = random_block(Some((parent_hash, i)));
        parent_hash = block.crypto_hash();
        chain.try_connect(block).await.unwrap();
    }

    assert_eq!(chain.current_header().await.unwrap().number(), times)
}

#[actix_rt::test]
async fn test_block_chain_forks() {
    //TODO
}
