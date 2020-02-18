use crate::BlockChain;
use anyhow::Result;
use crypto::{hash::CryptoHash, HashValue};
use std::collections::HashMap;
use types::block::{Block, BlockNumber};

///
/// Mock a unsafe block chain in memory
/// ```text
///   B0 --> B1 --> B2 --> B3 --> B4 --> B5
///                    |
///                 B2'└-> B3' -> B4' -> B5'
///                           |
///                           └-> B4"
/// ```
/// latest_block_number: 5
/// blocks: all block
/// indexes: block number to block
/// main_chain: B0 B1 B2 B3 B4 B5
pub struct MemChain {
    latest_block_number: BlockNumber,
    blocks: HashMap<HashValue, Block>,
    indexes: HashMap<BlockNumber, Vec<HashValue>>,
    main_chain: HashMap<BlockNumber, HashValue>,
}

impl MemChain {
    pub fn new(genesis_block: Block) -> Self {
        assert_eq!(genesis_block.header().number(), 0);

        let genesis_block_number = genesis_block.header().number();
        let genesis_block_hash = genesis_block.crypto_hash();
        let mut blocks = HashMap::new();
        blocks.insert(genesis_block_hash, genesis_block);
        let mut indexes = HashMap::new();
        let mut block_hash_vec = Vec::new();
        block_hash_vec.push(genesis_block_hash);
        indexes.insert(genesis_block_number, block_hash_vec);
        let mut main_chain = HashMap::new();
        main_chain.insert(genesis_block_number, genesis_block_hash);

        MemChain {
            latest_block_number: genesis_block_number,
            blocks,
            indexes,
            main_chain,
        }
    }
}

impl BlockChain for MemChain {
    fn get_block_by_hash(&self, hash: HashValue) -> Option<Block> {
        match self.blocks.get(&hash) {
            Some(b) => Some(b.clone()),
            None => None,
        }
    }

    fn try_connect(&mut self, block: Block) -> Result<()> {
        assert!((self.latest_block_number + 1) >= block.header().number());

        let block_hash = block.crypto_hash();
        let parent_hash = block.header().parent_hash();

        if !self.blocks.contains_key(&block_hash) && self.blocks.contains_key(&parent_hash) {
            assert_eq!(
                self.get_block_by_hash(parent_hash)
                    .expect("parent block is none.")
                    .header()
                    .number()
                    + 1,
                block.header().number()
            );

            if (self.latest_block_number + 1) == block.header().number() {
                //todo: rollback
                let block_hash_vec = Vec::new();
                let _ = self.indexes.insert(block.header().number(), block_hash_vec);
                let _ = self.main_chain.insert(block.header().number(), block_hash);
                self.latest_block_number = block.header().number();
            }

            self.indexes
                .get_mut(&block.header().number())
                .expect("index is none.")
                .push(block_hash);
            let _ = self.blocks.insert(block_hash, block);
        }

        Ok(())
    }
}
