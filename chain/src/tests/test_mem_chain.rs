use crate::mem_chain::MemChain;
use crate::BlockChain;
use crypto::{hash::CryptoHash, HashValue};
use types::block::{Block, BlockHeader, BlockNumber};

#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

fn random_block(parent_block: Option<(HashValue, BlockNumber)>) -> Block {
    let header = match parent_block {
        None => BlockHeader::genesis_block_header_for_test(),
        Some((parent_hash, parent_height)) => {
            BlockHeader::new_block_header_for_test(parent_hash, parent_height)
        }
    };

    Block::new_nil_block_for_test(header)
}

fn gen_mem_chain(times: u64) -> MemChain {
    let genesis_block = random_block(None);
    let mut latest_block_hash = genesis_block.crypto_hash();
    let mut chain = MemChain::new(genesis_block);

    for i in 0..times {
        let current_block = random_block(Some((latest_block_hash, i)));
        latest_block_hash = current_block.crypto_hash();
        chain.try_connect(current_block);
    }

    chain
}

#[test]
fn test_mem_chain() {
    let chain = gen_mem_chain(10);
}
