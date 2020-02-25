use super::random_block;
use crate::mem_chain::MemChain;
use crate::{ChainReader, ChainWriter};
use crypto::{hash::CryptoHash, HashValue};
use traits::ChainService;
use types::block::{Block, BlockHeader, BlockNumber};

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

pub fn gen_mem_chain_for_test(times: u64) -> MemChain {
    let genesis_block = random_block(None);
    let mut parent_block_hash = genesis_block.crypto_hash();
    let mut chain = MemChain::new(genesis_block);

    for i in 0..times {
        let current_block = random_block(Some((parent_block_hash, i)));
        parent_block_hash = current_block.crypto_hash();
        chain.try_connect(current_block);
    }

    chain
}

#[test]
fn test_mem_chain() {
    let chain = gen_mem_chain_for_test(10);
    println!("{}", chain.head_block().header().number());
    assert_eq!(chain.head_block().header().number(), 10);
}
