use crate::message::{ChainRequest, ChainResponse};
use actix::fut::wrap_future;
use actix::{Actor, Addr, Context, Handler, ResponseActFuture};
use anyhow::{Error, Result};
use config::NodeConfig;
use crypto::{hash::CryptoHash, HashValue};
use futures::compat::Future01CompatExt;
use futures_locks::RwLock;
use logger::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use traits::{ChainReader, ChainService, ChainStateReader, ChainWriter};
use types::{
    block::{Block, BlockHeader, BlockNumber, BlockTemplate},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};

pub struct MemChainActor {
    mem_chain: Arc<RwLock<MemChain>>,
}

impl MemChainActor {
    pub fn launch(
        // _node_config: &NodeConfig,
        genesis_block: Block,
    ) -> Result<Addr<MemChainActor>> {
        let actor = MemChainActor {
            mem_chain: Arc::new(RwLock::new(MemChain::new(genesis_block))),
        };
        Ok(actor.start())
    }
}

impl Actor for MemChainActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("ChainActor actor started");
    }
}

impl Handler<ChainRequest> for MemChainActor {
    type Result = ResponseActFuture<Self, Result<ChainResponse>>;

    fn handle(&mut self, msg: ChainRequest, ctx: &mut Self::Context) -> Self::Result {
        let mem_chain = self.mem_chain.clone();
        let fut = async move {
            match msg {
                ChainRequest::CreateBlock(times) => {
                    let mut lock = mem_chain.clone().write().compat().await.unwrap();
                    let head_block = lock.head_block().clone();
                    let mut parent_block_hash = head_block.header().id();
                    for i in 0..times {
                        debug!("parent_block_hash: {:?}", parent_block_hash);
                        let current_block_header =
                            BlockHeader::new_block_header_for_test(parent_block_hash, i);
                        let current_block = Block::new_nil_block_for_test(current_block_header);
                        parent_block_hash = current_block.header().id();
                        lock.try_connect(current_block);
                    }
                    Ok(ChainResponse::None)
                }
                ChainRequest::CurrentHeader() => {
                    let lock = mem_chain.clone().read().compat().await.unwrap();
                    Ok(ChainResponse::BlockHeader(lock.current_header()))
                }
                ChainRequest::GetHeaderByHash(hash) => {
                    let lock = mem_chain.clone().read().compat().await.unwrap();
                    Ok(ChainResponse::BlockHeader(
                        lock.get_header(hash).unwrap().unwrap(),
                    ))
                }
                ChainRequest::HeadBlock() => {
                    let lock = mem_chain.clone().read().compat().await.unwrap();
                    Ok(ChainResponse::Block(lock.head_block()))
                }
                ChainRequest::GetHeaderByNumber(number) => {
                    let lock = mem_chain.clone().read().compat().await.unwrap();
                    Ok(ChainResponse::BlockHeader(
                        lock.get_header_by_number(number).unwrap().unwrap(),
                    ))
                }
                ChainRequest::GetBlockByNumber(number) => {
                    let lock = mem_chain.clone().read().compat().await.unwrap();
                    Ok(ChainResponse::Block(
                        lock.get_block_by_number(number).unwrap().unwrap(),
                    ))
                }
                ChainRequest::CreateBlockTemplate() => {
                    let lock = mem_chain.clone().read().compat().await.unwrap();
                    Ok(ChainResponse::BlockTemplate(
                        lock.create_block_template(vec![]).unwrap(),
                    ))
                }
                ChainRequest::GetBlockByHash(hash) => {
                    let lock = mem_chain.clone().read().compat().await.unwrap();
                    Ok(ChainResponse::OptionBlock(lock.get_block(hash).unwrap()))
                }
                ChainRequest::ConnectBlock(block) => {
                    debug!("{:?}:{:?}", "connect block", block.header().id());
                    let mut lock = mem_chain.clone().write().compat().await.unwrap();
                    lock.try_connect(block).unwrap();
                    Ok(ChainResponse::None)
                }
                ChainRequest::GetHeadBranch() => {
                    let lock = mem_chain.clone().read().compat().await.unwrap();
                    let hash = lock.get_head_branch();
                    Ok(ChainResponse::HashValue(hash))
                }
            }
        };

        Box::new(wrap_future::<_, Self>(fut))
    }
}

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
        let genesis_block_hash = genesis_block.header().id();
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

    pub fn get_block_by_number_from_master(&self, number: &BlockNumber) -> Option<Block> {
        match self.master.get(number) {
            Some(hash) => Some(self.blocks.get(hash).expect("block is none.").clone()),
            None => None,
        }
    }
}

impl ChainService for MemChain {
    fn try_connect(&mut self, block: Block) -> Result<()> {
        assert!((self.head_number + 1) >= block.header().number());

        let block_hash = block.header().id();
        let parent_hash = block.header().parent_hash();

        if !self.blocks.contains_key(&block_hash) && self.blocks.contains_key(&parent_hash) {
            assert_eq!(
                self.get_block(parent_hash)
                    .unwrap()
                    .unwrap()
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

    fn get_head_branch(&self) -> HashValue {
        unimplemented!()
    }
}

impl ChainReader for MemChain {
    fn current_header(&self) -> BlockHeader {
        let head_block = self.head_block();
        head_block.header().clone()
    }

    fn head_block(&self) -> Block {
        let head_hash = self
            .master
            .get(&self.head_number)
            .expect("Get head block by head number none.");
        self.blocks
            .get(head_hash)
            .expect("Get block by hash none.")
            .clone()
    }

    fn get_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>> {
        Ok(self
            .get_block_by_number(number)?
            .map(|block| block.header().clone()))
    }

    fn get_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        let hash = self.master.get(&number).expect("hash is none.");
        Ok(self.blocks.get(hash).cloned())
    }

    fn create_block_template(&self, _txns: Vec<SignedUserTransaction>) -> Result<BlockTemplate> {
        let head_block = self.head_block().clone();
        let head_block_hash = head_block.header().id();
        let block_template_header =
            BlockHeader::new_block_header_for_test(head_block_hash, head_block.header().number());
        let current_block = Block::new_nil_block_for_test(block_template_header);
        Ok(BlockTemplate::from_block(current_block))
    }

    fn get_header(&self, hash: HashValue) -> Result<Option<BlockHeader>> {
        Ok(self.get_block(hash)?.map(|block| block.header().clone()))
    }

    fn get_block(&self, hash: HashValue) -> Result<Option<Block>> {
        Ok(match self.blocks.get(&hash) {
            Some(block) => Some(block.clone()),
            None => None,
        })
    }

    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>> {
        unimplemented!()
    }

    fn get_transaction_info(&self, hash: HashValue) -> Result<Option<TransactionInfo>> {
        unimplemented!()
    }

    fn chain_state_reader(&self) -> &ChainStateReader {
        unimplemented!()
    }
}
