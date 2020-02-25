use crypto::HashValue;
use types::block::{Block, BlockHeader, BlockNumber};

mod test_block_chain;
mod test_mem_chain;

pub use test_mem_chain::gen_mem_chain_for_test;

fn random_block(parent_block: Option<(HashValue, BlockNumber)>) -> Block {
    let header = match parent_block {
        None => BlockHeader::genesis_block_header_for_test(),
        Some((parent_hash, parent_number)) => {
            BlockHeader::new_block_header_for_test(parent_hash, parent_number)
        }
    };

    Block::new_nil_block_for_test(header)
}
