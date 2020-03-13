use crypto::HashValue;
use types::block::{Block, BlockHeader, BlockNumber};

#[cfg(test)]
mod test_block_chain;

fn random_block(parent_block: Option<(HashValue, BlockNumber)>) -> Block {
    let header = match parent_block {
        None => BlockHeader::genesis_block_header_for_test(),
        Some((parent_hash, parent_number)) => {
            BlockHeader::new_block_header_for_test(parent_hash, parent_number)
        }
    };

    Block::new_nil_block_for_test(header)
}
