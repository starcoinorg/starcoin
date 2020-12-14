// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::message::{ChainRequest, ChainResponse};
use anyhow::{bail, Result};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::block::{BlockState, BlockSummary, EpochUncleSummary};
use starcoin_types::contract_event::{ContractEvent, ContractEventInfo};
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::ChainStatus;
use starcoin_types::stress_test::TPS;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    startup_info::StartupInfo,
};
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};

/// Readable block chain service trait
pub trait ReadableChainService {
    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>>;
    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>>;
    fn get_headers(&self, ids: Vec<HashValue>) -> Result<Vec<BlockHeader>>;
    fn get_block_state_by_hash(&self, hash: HashValue) -> Result<Option<BlockState>>;
    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>>;
    fn get_transaction_block_hash(&self, txn_hash: HashValue) -> Result<Option<HashValue>>;
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>>;
    fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<TransactionInfo>>;
    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>>;
    fn get_events_by_txn_info_hash(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>>;
    /// for main
    fn main_head_header(&self) -> BlockHeader;
    fn main_head_block(&self) -> Block;
    fn main_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    fn main_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>>;
    fn main_block_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn main_startup_info(&self) -> StartupInfo;
    fn main_blocks_by_number(&self, number: Option<BlockNumber>, count: u64) -> Result<Vec<Block>>;
    fn epoch_info(&self) -> Result<EpochInfo>;
    fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo>;
    fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain>;
    fn get_main_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>>;
    fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>>;
    fn tps(&self, number: Option<BlockNumber>) -> Result<TPS>;
    fn get_epoch_uncles_by_number(&self, number: Option<BlockNumber>) -> Result<Vec<BlockSummary>>;
    fn uncle_path(&self, block_id: HashValue, uncle_id: HashValue) -> Result<Vec<BlockHeader>>;
    fn epoch_uncle_summary_by_number(
        &self,
        number: Option<BlockNumber>,
    ) -> Result<EpochUncleSummary>;
}

/// Writeable block chain service trait
pub trait WriteableChainService: Send + Sync {
    fn try_connect(&mut self, block: Block) -> Result<()>;
}

#[async_trait::async_trait]
pub trait ChainAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    async fn get_header_by_hash(&self, hash: &HashValue) -> Result<Option<BlockHeader>>;
    async fn get_block_by_hash(&self, hash: HashValue) -> Result<Block>;
    async fn get_blocks(&self, hashes: Vec<HashValue>) -> Result<Vec<Option<Block>>>;
    async fn get_headers(&self, hashes: Vec<HashValue>) -> Result<Vec<BlockHeader>>;
    async fn uncle_path(
        &self,
        block_id: HashValue,
        uncle_id: HashValue,
    ) -> Result<Vec<BlockHeader>>;
    async fn get_block_state_by_hash(&self, hash: &HashValue) -> Result<Option<BlockState>>;
    async fn get_block_info_by_hash(&self, hash: &HashValue) -> Result<Option<BlockInfo>>;
    async fn get_transaction(&self, txn_hash: HashValue) -> Result<Transaction>;
    async fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<TransactionInfo>>;
    async fn get_transaction_block(&self, txn_hash: HashValue) -> Result<Option<Block>>;
    async fn get_block_txn_infos(&self, block_hash: HashValue) -> Result<Vec<TransactionInfo>>;
    async fn get_txn_info_by_block_and_index(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfo>>;
    async fn get_events_by_txn_hash(&self, txn_hash: HashValue) -> Result<Vec<ContractEventInfo>>;
    /// for main
    async fn main_head_header(&self) -> Result<BlockHeader>;
    async fn main_head_block(&self) -> Result<Block>;
    async fn main_block_by_number(&self, number: BlockNumber) -> Result<Block>;
    async fn main_block_by_uncle(&self, uncle_hash: HashValue) -> Result<Option<Block>>;
    async fn main_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>>;
    async fn main_block_header_by_number(&self, number: BlockNumber) -> Result<BlockHeader>;
    async fn main_startup_info(&self) -> Result<StartupInfo>;
    async fn main_status(&self) -> Result<ChainStatus>;
    async fn epoch_info(&self) -> Result<EpochInfo>;
    async fn get_epoch_info_by_number(&self, number: BlockNumber) -> Result<EpochInfo>;
    async fn get_global_time_by_number(&self, number: BlockNumber) -> Result<GlobalTimeOnChain>;
    async fn main_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>>;
    async fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>>;
    async fn tps(&self, number: Option<BlockNumber>) -> Result<TPS>;
    async fn get_epoch_uncles_by_number(
        &self,
        number: Option<BlockNumber>,
    ) -> Result<Vec<BlockSummary>>;
    async fn epoch_uncle_summary_by_number(
        &self,
        number: Option<BlockNumber>,
    ) -> Result<EpochUncleSummary>;
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

    async fn get_blocks(&self, hashes: Vec<HashValue>) -> Result<Vec<Option<Block>>> {
        if let ChainResponse::BlockOptionVec(blocks) =
            self.send(ChainRequest::GetBlocks(hashes)).await??
        {
            Ok(blocks)
        } else {
            bail!("get_blocks response type error.")
        }
    }

    async fn get_headers(&self, ids: Vec<HashValue>) -> Result<Vec<BlockHeader>> {
        if let ChainResponse::BlockHeaderVec(headers) =
            self.send(ChainRequest::GetHeaders(ids)).await??
        {
            Ok(headers)
        } else {
            bail!("get_headers response type error.")
        }
    }

    async fn get_block_state_by_hash(&self, hash: &HashValue) -> Result<Option<BlockState>> {
        if let ChainResponse::BlockState(block_state) = self
            .send(ChainRequest::GetBlockStateByHash(*hash))
            .await??
        {
            Ok(block_state.map(|block| *block))
        } else {
            bail!("get_block_state_by_hash response type error")
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

    async fn get_transaction_block(&self, txn_hash: HashValue) -> Result<Option<Block>> {
        let response = self
            .send(ChainRequest::GetTransactionBlock(txn_hash))
            .await??;
        if let ChainResponse::BlockOption(b) = response {
            Ok(b.map(|d| *d))
        } else {
            bail!("get transaction_block error:{:?}", txn_hash)
        }
    }

    async fn get_block_txn_infos(&self, block_hash: HashValue) -> Result<Vec<TransactionInfo>> {
        let response = self
            .send(ChainRequest::GetBlockTransactionInfos(block_hash))
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
                block_hash: block_id,
                txn_idx: idx,
            })
            .await??;
        if let ChainResponse::TransactionInfo(info) = response {
            Ok(info)
        } else {
            bail!("get txn info by block and idx error.")
        }
    }
    async fn get_events_by_txn_hash(&self, txn_hash: HashValue) -> Result<Vec<ContractEventInfo>> {
        let response = self
            .send(ChainRequest::GetEventsByTxnHash { txn_hash })
            .await??;
        if let ChainResponse::Events(events) = response {
            Ok(events)
        } else {
            bail!("get txn info by block and idx error.")
        }
    }

    async fn main_head_header(&self) -> Result<BlockHeader> {
        if let ChainResponse::BlockHeader(header) =
            self.send(ChainRequest::CurrentHeader()).await??
        {
            Ok(*header)
        } else {
            bail!("Get main head header response error.")
        }
    }

    async fn main_head_block(&self) -> Result<Block> {
        if let ChainResponse::Block(block) = self.send(ChainRequest::HeadBlock()).await?? {
            Ok(*block)
        } else {
            bail!("Get main head block response error.")
        }
    }

    async fn main_block_by_number(&self, number: BlockNumber) -> Result<Block> {
        if let ChainResponse::Block(block) =
            self.send(ChainRequest::GetBlockByNumber(number)).await??
        {
            Ok(*block)
        } else {
            bail!("Get chain block by number response error.")
        }
    }

    async fn main_block_by_uncle(&self, uncle_id: HashValue) -> Result<Option<Block>> {
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

    async fn main_blocks_by_number(
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

    async fn main_block_header_by_number(&self, number: BlockNumber) -> Result<BlockHeader> {
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

    async fn main_startup_info(&self) -> Result<StartupInfo> {
        let response = self.send(ChainRequest::GetStartupInfo()).await??;
        if let ChainResponse::StartupInfo(startup_info) = response {
            Ok(*startup_info)
        } else {
            bail!("Get chain info response error.")
        }
    }

    async fn main_status(&self) -> Result<ChainStatus> {
        let response = self.send(ChainRequest::GetHeadChainStatus()).await??;
        if let ChainResponse::ChainStatus(chain_status) = response {
            Ok(*chain_status)
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
    async fn main_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>> {
        let response = self.send(ChainRequest::MainEvents(filter)).await??;
        if let ChainResponse::MainEvents(evts) = response {
            Ok(evts)
        } else {
            bail!("get main events error.")
        }
    }

    async fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
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

    async fn tps(&self, number: Option<BlockNumber>) -> Result<TPS> {
        let response = self.send(ChainRequest::TPS(number)).await??;
        if let ChainResponse::TPS(tps) = response {
            Ok(tps)
        } else {
            bail!("get tps error.")
        }
    }

    async fn get_epoch_uncles_by_number(
        &self,
        number: Option<BlockNumber>,
    ) -> Result<Vec<BlockSummary>> {
        let response = self
            .send(ChainRequest::GetEpochUnclesByNumber(number))
            .await??;
        if let ChainResponse::BlockSummaries(summaries) = response {
            Ok(summaries)
        } else {
            bail!("get epoch uncles error.")
        }
    }

    async fn epoch_uncle_summary_by_number(
        &self,
        number: Option<BlockNumber>,
    ) -> Result<EpochUncleSummary> {
        let response = self
            .send(ChainRequest::EpochUncleSummaryByNumber(number))
            .await??;
        if let ChainResponse::UncleSummary(summary) = response {
            Ok(summary)
        } else {
            bail!("epoch uncle summary error.")
        }
    }

    async fn uncle_path(
        &self,
        block_id: HashValue,
        uncle_id: HashValue,
    ) -> Result<Vec<BlockHeader>> {
        let response = self
            .send(ChainRequest::UnclePath(block_id, uncle_id))
            .await??;
        if let ChainResponse::BlockHeaderVec(headers) = response {
            Ok(headers)
        } else {
            bail!("get uncle path error.")
        }
    }
}
