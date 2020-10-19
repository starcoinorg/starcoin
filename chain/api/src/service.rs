// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::message::{ChainRequest, ChainResponse};
use anyhow::{bail, Result};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::block::BlockState;
use starcoin_types::contract_event::{ContractEvent, ContractEventInfo};
use starcoin_types::filter::Filter;
use starcoin_types::peer_info::PeerId;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    startup_info::StartupInfo,
};
use starcoin_vm_types::on_chain_config::{EpochInfo, GlobalTimeOnChain};

/// Readable block chain service trait
pub trait ReadableChainService {
    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>>;
    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>>;
    fn get_block_state_by_hash(&self, hash: HashValue) -> Result<Option<BlockState>>;
    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>>;
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>>;
    fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>>;
    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>>;
    fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>>;
    /// for master
    fn master_head_header(&self) -> BlockHeader;
    fn master_head_block(&self) -> Block;
    fn master_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    fn master_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>>;
    fn master_block_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn master_startup_info(&self) -> StartupInfo;
    fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>>;
    fn epoch_info(&self) -> Result<EpochInfo>;
    fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo>;
    fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain>;
    fn get_master_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>>;
    fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: usize,
    ) -> Result<Vec<HashValue>>;
}

/// Writeable block chain service trait
pub trait WriteableChainService: Send + Sync {
    fn try_connect(&mut self, block: Block) -> Result<()>;

    fn try_connect_without_execute(
        &mut self,
        block: Block,
        remote_peer_id_to_read_state: PeerId,
    ) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ChainAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    async fn get_header_by_hash(&self, hash: &HashValue) -> Result<Option<BlockHeader>>;
    async fn get_block_by_hash(&self, hash: HashValue) -> Result<Block>;
    async fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>>;
    async fn get_block_state_by_hash(&self, hash: &HashValue) -> Result<Option<BlockState>>;
    async fn get_block_info_by_hash(&self, hash: &HashValue) -> Result<Option<BlockInfo>>;
    async fn get_transaction(&self, txn_hash: HashValue) -> Result<Transaction>;
    async fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>>;
    async fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>>;
    async fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>>;
    async fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>>;
    /// for master
    async fn master_head_header(&self) -> Result<BlockHeader>;
    async fn master_head_block(&self) -> Result<Block>;
    async fn master_block_by_number(&self, number: BlockNumber) -> Result<Block>;
    async fn master_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>>;
    async fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>>;
    async fn master_block_header_by_number(&self, number: BlockNumber) -> Result<BlockHeader>;
    async fn master_startup_info(&self) -> Result<StartupInfo>;
    async fn master_head(&self) -> Result<ChainInfo>;
    async fn epoch_info(&self) -> Result<EpochInfo>;
    async fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo>;
    async fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain>;
    async fn master_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>>;
    async fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: usize,
    ) -> Result<Vec<HashValue>>;
}

#[async_trait::async_trait]
impl<S> ChainAsyncService for ServiceRef<S>
where
    S: ActorService + ServiceHandler<S, ChainRequest>,
{
    async fn get_header_by_hash(&self, hash: &HashValue) -> Result<Option<BlockHeader>> {
        if let ChainResponse::BlockHeaderOption(header) =
            self.send(ChainRequest::GetHeaderByHash(*hash)).await??
        {
            if let Some(h) = *header {
                return Ok(Some(h));
            }
        }
        Ok(None)
    }

    async fn get_block_by_hash(&self, hash: HashValue) -> Result<Block> {
        if let ChainResponse::BlockOption(block) =
            self.send(ChainRequest::GetBlockByHash(hash)).await??
        {
            match block {
                Some(b) => Ok(*b),
                None => bail!("get block by hash is none: {:?}", hash),
            }
        } else {
            bail!("get block by hash error.")
        }
    }

    async fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>> {
        if let ChainResponse::BlockOptionVec(blocks) =
            self.send(ChainRequest::GetBlocks(ids)).await??
        {
            Ok(blocks)
        } else {
            bail!("get blocks error.")
        }
    }

    async fn get_block_state_by_hash(&self, hash: &HashValue) -> Result<Option<BlockState>> {
        if let ChainResponse::BlockState(Some(block_state)) = self
            .send(ChainRequest::GetBlockStateByHash(*hash))
            .await??
        {
            Ok(Some(*block_state))
        } else {
            bail!("get block state by hash error.")
        }
    }

    async fn get_block_info_by_hash(&self, hash: &HashValue) -> Result<Option<BlockInfo>> {
        if let ChainResponse::BlockInfoOption(block_info) =
            self.send(ChainRequest::GetBlockInfoByHash(*hash)).await??
        {
            return Ok(*block_info);
        }
        Ok(None)
    }

    async fn get_transaction(&self, txn_hash: HashValue) -> Result<Transaction> {
        let response = self.send(ChainRequest::GetTransaction(txn_hash)).await??;
        if let ChainResponse::Transaction(txn) = response {
            Ok(*txn)
        } else {
            bail!("get transaction error.")
        }
    }

    async fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>> {
        let response = self
            .send(ChainRequest::GetTransactionInfo(txn_hash))
            .await??;
        if let ChainResponse::TransactionInfo(txn_info) = response {
            Ok(txn_info)
        } else {
            bail!("get transaction_info error:{:?}", txn_hash)
        }
    }

    async fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>> {
        let response = self
            .send(ChainRequest::GetBlockTransactionInfos(block_id))
            .await??;
        if let ChainResponse::TransactionInfos(vec_txn_id) = response {
            Ok(vec_txn_id)
        } else {
            bail!("get block's transaction_info error.")
        }
    }

    async fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>> {
        let response = self
            .send(ChainRequest::GetTransactionInfoByBlockAndIndex {
                block_id,
                txn_idx: idx,
            })
            .await??;
        if let ChainResponse::TransactionInfo(info) = response {
            Ok(info)
        } else {
            bail!("get txn info by block and idx error.")
        }
    }
    async fn get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>> {
        let response = self
            .send(ChainRequest::GetEventsByTxnInfoId { txn_info_id })
            .await??;
        if let ChainResponse::Events(events) = response {
            Ok(events)
        } else {
            bail!("get txn info by block and idx error.")
        }
    }

    async fn master_head_header(&self) -> Result<BlockHeader> {
        if let ChainResponse::BlockHeader(header) =
            self.send(ChainRequest::CurrentHeader()).await??
        {
            Ok(*header)
        } else {
            bail!("Get master head header response error.")
        }
    }

    async fn master_head_block(&self) -> Result<Block> {
        if let ChainResponse::Block(block) = self.send(ChainRequest::HeadBlock()).await?? {
            Ok(*block)
        } else {
            bail!("Get master head block response error.")
        }
    }

    async fn master_block_by_number(&self, number: BlockNumber) -> Result<Block> {
        if let ChainResponse::Block(block) =
            self.send(ChainRequest::GetBlockByNumber(number)).await??
        {
            Ok(*block)
        } else {
            bail!("Get chain block by number response error.")
        }
    }

    async fn master_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>> {
        if let ChainResponse::BlockOption(block) =
            self.send(ChainRequest::GetBlockByUncle(uncle_id)).await??
        {
            match block {
                Some(b) => Ok(Some(*b)),
                None => Ok(None),
            }
        } else {
            bail!("get block by hash error.")
        }
    }

    async fn master_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>> {
        if let ChainResponse::BlockVec(blocks) = self
            .send(ChainRequest::GetBlocksByNumber(number, count))
            .await??
        {
            Ok(blocks)
        } else {
            bail!("Get chain blocks by number response error.")
        }
    }

    async fn master_block_header_by_number(&self, number: BlockNumber) -> Result<BlockHeader> {
        if let ChainResponse::BlockHeaderOption(header) = self
            .send(ChainRequest::GetBlockHeaderByNumber(number))
            .await??
        {
            if let Some(h) = *header {
                return Ok(h);
            }
        }
        bail!("Get chain block header by number response error.")
    }

    async fn master_startup_info(&self) -> Result<StartupInfo> {
        let response = self.send(ChainRequest::GetStartupInfo()).await??;
        if let ChainResponse::StartupInfo(startup_info) = response {
            Ok(startup_info)
        } else {
            bail!("Get chain info response error.")
        }
    }

    async fn master_head(&self) -> Result<ChainInfo> {
        let response = self.send(ChainRequest::GetHeadChainInfo()).await??;
        if let ChainResponse::ChainInfo(chain_info) = response {
            Ok(chain_info)
        } else {
            bail!("get head chain info error.")
        }
    }

    async fn epoch_info(&self) -> Result<EpochInfo> {
        let response = self.send(ChainRequest::GetEpochInfo()).await??;
        if let ChainResponse::EpochInfo(epoch_info) = response {
            Ok(epoch_info)
        } else {
            bail!("get epoch chain info error.")
        }
    }

    async fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo> {
        let response = self
            .send(ChainRequest::GetEpochInfoByNumber(number))
            .await??;
        if let ChainResponse::EpochInfo(epoch_info) = response {
            Ok(epoch_info)
        } else {
            bail!("get epoch chain info error.")
        }
    }

    async fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain> {
        let response = self
            .send(ChainRequest::GetGlobalTimeByNumber(number))
            .await??;
        if let ChainResponse::GlobalTime(global_time) = response {
            Ok(global_time)
        } else {
            bail!("get global time error.")
        }
    }
    async fn master_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>> {
        let response = self.send(ChainRequest::MasterEvents(filter)).await??;
        if let ChainResponse::MasterEvents(evts) = response {
            Ok(evts)
        } else {
            bail!("get master events error.")
        }
    }

    async fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: usize,
    ) -> Result<Vec<HashValue>> {
        let response = self
            .send(ChainRequest::GetBlockIds {
                start_number,
                reverse,
                max_size,
            })
            .await??;
        if let ChainResponse::HashVec(ids) = response {
            Ok(ids)
        } else {
            bail!("get_block_ids invalid response")
        }
    }
}
