use crate::BlockChain;
use anyhow::Result;
use crypto::{hash::CryptoHash, HashValue};
use std::collections::HashMap;
use types::block::{Block, BlockHeader, BlockNumber};

///
/// unsafe block chain in memory
/// ```text
///   B0 --> B1 --> B2 --> B3 --> B4 --> B5
///                    |
///                 B2'└-> B3' -> B4' -> B5'
///                           |
///                           └-> B4"
/// ```
/// head_number: 5
/// blocks: all block
/// indexes: block number to block
/// master: B0 B1 B2 B3 B4 B5
pub struct MemChain {
    head_number: BlockNumber,
    blocks: HashMap<HashValue, Block>,
    indexes: HashMap<BlockNumber, Vec<HashValue>>,
    master: HashMap<BlockNumber, HashValue>,
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
        let mut master = HashMap::new();
        master.insert(genesis_block_number, genesis_block_hash);

        MemChain {
            head_number: genesis_block_number,
            blocks,
            indexes,
            master,
        }
    }

    pub fn head_block(&self) -> &Block {
        let head_hash = self
            .master
            .get(&self.head_number)
            .expect("Get head block by head number none.");
        self.blocks.get(head_hash).expect("Get block by hash none.")
    }

    pub fn current_header(&self) -> BlockHeader {
        let head_block = self.head_block();
        head_block.header().clone()
    }

    pub fn get_block_by_number_from_master(&self, number: &BlockNumber) -> Option<Block> {
        match self.master.get(number) {
            Some(hash) => Some(self.blocks.get(hash).expect("block is none.").clone()),
            None => None,
        }
    }
}

impl BlockChain for MemChain {
    fn get_block_by_hash(&self, hash: &HashValue) -> Option<Block> {
        match self.blocks.get(hash) {
            Some(b) => Some(b.clone()),
            None => None,
        }
    }

    fn try_connect(&mut self, block: Block) -> Result<()> {
        assert!((self.head_number + 1) >= block.header().number());

        let block_hash = block.crypto_hash();
        let parent_hash = block.header().parent_hash();

        if !self.blocks.contains_key(&block_hash) && self.blocks.contains_key(&parent_hash) {
            assert_eq!(
                self.get_block_by_hash(&parent_hash)
                    .expect("parent block is none.")
                    .header()
                    .number()
                    + 1,
                block.header().number()
            );

            if (self.head_number + 1) == block.header().number() {
                //todo: rollback
                let block_hash_vec = Vec::new();
                let _ = self.indexes.insert(block.header().number(), block_hash_vec);
                let _ = self.master.insert(block.header().number(), block_hash);
                self.head_number = block.header().number();
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
