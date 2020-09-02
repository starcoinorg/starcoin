use crate::block_connector::FutureBlockPool;
use crypto::HashValue;
use types::block::{Block, BlockBody, BlockHeader};

fn _gen_future_block_pool(block_ct: usize) -> (FutureBlockPool, HashValue, Vec<HashValue>) {
    let pool = FutureBlockPool::new();
    let mut blocks = Vec::new();
    let mut parent_id = None;
    let mut genesis_id = None;
    for _i in 0..block_ct {
        let mut block = Block::new(BlockHeader::random(), BlockBody::new_empty());
        match parent_id {
            Some(id) => block.header.parent_hash = id,
            None => genesis_id = Some(block.header().parent_hash()),
        }

        blocks.push(block.id());
        parent_id = Some(block.id());
        pool.add_future_block(block);
    }

    (pool, genesis_id.unwrap(), blocks)
}

#[stest::test]
fn future_block_pool() {
    let (pool, genesis_id, mut blocks) = _gen_future_block_pool(1);
    assert_eq!(blocks.len(), 1);
    let grandpa_id = genesis_id;
    let parent_id = blocks.remove(0);
    assert_eq!(pool._len(), 1);
    let mut child_block = Block::new(BlockHeader::random(), BlockBody::new_empty());
    let old_id = child_block.id();
    child_block.header.parent_hash = parent_id;
    let new_id = child_block.id();
    assert_ne!(old_id, new_id);
    pool.add_future_block(child_block);
    assert_eq!(pool._len(), 2);

    assert_eq!(pool._son_len(&parent_id), 1);
    let blocks_1 = pool.take_child(&parent_id).unwrap();
    assert_eq!(blocks_1.len(), 1);
    assert_eq!(pool._son_len(&parent_id), 0);
    assert_eq!(pool._len(), 1);

    assert_eq!(pool._son_len(&grandpa_id), 1);

    let blocks_2 = pool.take_child(&grandpa_id).unwrap();
    assert_eq!(blocks_2.len(), 1);
    assert_eq!(pool._son_len(&grandpa_id), 0);
    assert_eq!(pool._len(), 0);
}

#[stest::test]
fn future_block_pool_take_child() {
    let (pool, genesis_id, _) = _gen_future_block_pool(3);
    let blocks = pool.take_child(&genesis_id).unwrap();
    assert_eq!(blocks.len(), 3);
    assert_eq!(pool._len(), 0);
}
