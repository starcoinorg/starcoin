use actix::prelude::*;
use actix::{Actor, Addr, Context};
use anyhow::Result;
use bus::{BusActor, Subscription};
use chain::BlockChain;
use consensus::Consensus;
use crypto::hash::HashValue;
use logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_vm_types::genesis_config::{ChainNetwork, ConsensusStrategy};
use std::{collections::HashMap, sync::Arc};
use storage::Store;
use traits::{ChainReader, ExcludedTxns};
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockTemplate},
    system_events::{NewBranch, NewHeadBlock},
    transaction::SignedUserTransaction,
};

const MAX_UNCLE_COUNT_PER_BLOCK: usize = 2;

pub type CreateBlockTemplateActorAddress = Addr<CreateBlockTemplateActor>;

pub struct GetHeadRequest;

pub struct GetHeadResponse {
    pub head: HashValue,
}

impl Message for GetHeadRequest {
    type Result = Result<GetHeadResponse>;
}

pub struct CreateBlockTemplateRequest {
    final_block_gas_limit: u64,
    author: AccountAddress,
    auth_key_prefix: Option<Vec<u8>>,
    user_txns: Vec<SignedUserTransaction>,
}

impl CreateBlockTemplateRequest {
    pub fn new(
        final_block_gas_limit: u64,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Self {
        Self {
            final_block_gas_limit,
            author,
            auth_key_prefix,
            user_txns,
        }
    }
}

impl
    Into<(
        u64,
        AccountAddress,
        Option<Vec<u8>>,
        Vec<SignedUserTransaction>,
    )> for CreateBlockTemplateRequest
{
    fn into(
        self,
    ) -> (
        u64,
        AccountAddress,
        Option<Vec<u8>>,
        Vec<SignedUserTransaction>,
    ) {
        (
            self.final_block_gas_limit,
            self.author,
            self.auth_key_prefix,
            self.user_txns,
        )
    }
}

pub struct CreateBlockTemplateResponse {
    block_template: BlockTemplate,
    txns: ExcludedTxns,
}

impl Into<(BlockTemplate, ExcludedTxns)> for CreateBlockTemplateResponse {
    fn into(self) -> (BlockTemplate, ExcludedTxns) {
        (self.block_template, self.txns)
    }
}

impl Message for CreateBlockTemplateRequest {
    type Result = Result<CreateBlockTemplateResponse>;
}

pub struct CreateBlockTemplateActor {
    bus: Addr<BusActor>,
    inner: Inner,
}

impl CreateBlockTemplateActor {
    pub fn launch(
        block_id: HashValue,
        net: &ChainNetwork,
        bus: Addr<BusActor>,
        storage: Arc<dyn Store>,
    ) -> Result<CreateBlockTemplateActorAddress> {
        let inner = Inner::new(block_id, storage, net)?;
        Ok(CreateBlockTemplateActor::create(move |_ctx| {
            CreateBlockTemplateActor { bus, inner }
        }))
    }
}

impl Actor for CreateBlockTemplateActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<NewHeadBlock>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        let recipient = ctx.address().recipient::<NewBranch>();
        self.bus
            .send(Subscription { recipient })
            .into_actor(self)
            .then(|_res, act, _ctx| async {}.into_actor(act))
            .wait(ctx);

        info!("CreateBlockTemplateActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("CreateBlockTemplateActor stopped");
    }
}

impl Handler<NewHeadBlock> for CreateBlockTemplateActor {
    type Result = ();

    fn handle(&mut self, msg: NewHeadBlock, _ctx: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.inner.update_chain(msg.0.get_block().clone()) {
            error!("err : {:?}", e);
        }
    }
}

impl Handler<NewBranch> for CreateBlockTemplateActor {
    type Result = ();

    fn handle(&mut self, msg: NewBranch, _ctx: &mut Self::Context) -> Self::Result {
        self.inner.insert_uncle((&*msg.0).clone());
    }
}

impl Handler<CreateBlockTemplateRequest> for CreateBlockTemplateActor {
    type Result = Result<CreateBlockTemplateResponse>;

    fn handle(
        &mut self,
        msg: CreateBlockTemplateRequest,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let (final_block_gas_limit, author, auth_key_prefix, user_txns) = msg.into();
        let (block_template, txns) = self.inner.create_block_template(
            final_block_gas_limit,
            author,
            auth_key_prefix,
            user_txns,
        )?;
        Ok(CreateBlockTemplateResponse {
            block_template,
            txns,
        })
    }
}

impl Handler<GetHeadRequest> for CreateBlockTemplateActor {
    type Result = Result<GetHeadResponse>;

    fn handle(&mut self, _msg: GetHeadRequest, _ctx: &mut Self::Context) -> Self::Result {
        Ok(GetHeadResponse {
            head: self.inner.chain.current_header().id(),
        })
    }
}

pub struct Inner {
    chain: BlockChain,
    parent_uncle: HashMap<HashValue, Vec<HashValue>>,
    uncles: HashMap<HashValue, BlockHeader>,
    storage: Arc<dyn Store>,
    consensus: ConsensusStrategy,
}

impl Inner {
    fn insert_uncle(&mut self, uncle: BlockHeader) {
        self.parent_uncle
            .entry(uncle.parent_hash())
            .or_insert_with(Vec::new)
            .push(uncle.id());
        self.uncles.insert(uncle.id(), uncle);
    }

    fn new(block_id: HashValue, storage: Arc<dyn Store>, net: &ChainNetwork) -> Result<Self> {
        let chain = BlockChain::new(net.consensus(), block_id, storage.clone(), None)?;

        Ok(Inner {
            chain,
            parent_uncle: HashMap::new(),
            uncles: HashMap::new(),
            storage,
            consensus: net.consensus(),
        })
    }

    fn update_chain(&mut self, block: Block) -> Result<()> {
        if block.header().parent_hash() != self.chain.current_header().id() {
            self.chain = BlockChain::new(self.consensus, block.id(), self.storage.clone(), None)?;
        } else {
            self.chain.update_chain_head(block)?;
        }
        // TODO:prune uncles when switch epoch
        Ok(())
    }

    fn do_uncles(&self) -> Vec<BlockHeader> {
        let mut new_uncle = Vec::new();
        for maybe_uncle in self.uncles.values() {
            if new_uncle.len() >= MAX_UNCLE_COUNT_PER_BLOCK {
                break;
            }
            if self.chain.can_be_uncle(maybe_uncle) {
                new_uncle.push(maybe_uncle.clone())
            }
        }

        new_uncle
    }

    pub fn create_block_template(
        &self,
        final_block_gas_limit: u64,
        author: AccountAddress,
        auth_key_prefix: Option<Vec<u8>>,
        user_txns: Vec<SignedUserTransaction>,
    ) -> Result<(BlockTemplate, ExcludedTxns)> {
        let previous_header = self.chain.current_header();
        let uncles = self.do_uncles();
        let mut opened_block = OpenedBlock::new(
            self.storage.clone(),
            previous_header,
            final_block_gas_limit,
            author,
            auth_key_prefix,
            self.consensus.now(),
            uncles,
        )?;
        let excluded_txns = opened_block.push_txns(user_txns)?;
        let template = opened_block.finalize()?;
        Ok((template, excluded_txns))
    }
}
