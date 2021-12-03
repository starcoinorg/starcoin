// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::message::{ChainRequest, ChainResponse};
use crate::TransactionInfoWithProof;
use anyhow::{bail, Result};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::contract_event::{ContractEvent, ContractEventInfo};
use starcoin_types::filter::Filter;
use starcoin_types::startup_info::ChainStatus;
use starcoin_types::transaction::{RichTransactionInfo, Transaction};
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    startup_info::StartupInfo,
};
use starcoin_vm_types::access_path::AccessPath;

/// Readable block chain service trait
pub trait ReadableChainService {
    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>>;
    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>>;
    fn get_headers(&self, ids: Vec<HashValue>) -> Result<Vec<BlockHeader>>;
    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<Transaction>>;
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<RichTransactionInfo>>;
    fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<RichTransactionInfo>>;
    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<RichTransactionInfo>>;
    fn get_events_by_txn_info_hash(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>>;
    /// for main
    fn main_head_header(&self) -> BlockHeader;
    fn main_head_block(&self) -> Block;
    fn main_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    fn main_block_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn main_block_info_by_number(&self, number: BlockNumber) -> Result<Option<BlockInfo>>;
    fn main_startup_info(&self) -> StartupInfo;
    fn main_blocks_by_number(&self, number: Option<BlockNumber>, count: u64) -> Result<Vec<Block>>;
    fn get_main_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>>;
    fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>>;

    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<RichTransactionInfo>>;

    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>>;
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
    async fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>>;
    async fn get_blocks(&self, hashes: Vec<HashValue>) -> Result<Vec<Option<Block>>>;
    async fn get_headers(&self, hashes: Vec<HashValue>) -> Result<Vec<BlockHeader>>;
    async fn get_block_info_by_hash(&self, hash: &HashValue) -> Result<Option<BlockInfo>>;
    async fn get_block_info_by_number(&self, number: u64) -> Result<Option<BlockInfo>>;
    async fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>>;
    async fn get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> Result<Option<RichTransactionInfo>>;
    async fn get_transaction_block(&self, txn_hash: HashValue) -> Result<Option<Block>>;
    async fn get_block_txn_infos(&self, block_hash: HashValue) -> Result<Vec<RichTransactionInfo>>;
    async fn get_txn_info_by_block_and_index(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> Result<Option<RichTransactionInfo>>;
    async fn get_events_by_txn_hash(&self, txn_hash: HashValue) -> Result<Vec<ContractEventInfo>>;
    /// for main
    async fn main_head_header(&self) -> Result<BlockHeader>;
    async fn main_head_block(&self) -> Result<Block>;
    async fn main_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    async fn main_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> Result<Vec<Block>>;
    async fn main_block_header_by_number(&self, number: BlockNumber)
        -> Result<Option<BlockHeader>>;
    async fn main_startup_info(&self) -> Result<StartupInfo>;
    async fn main_status(&self) -> Result<ChainStatus>;
    async fn main_events(&self, filter: Filter) -> Result<Vec<ContractEventInfo>>;
    async fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<HashValue>>;
    async fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<RichTransactionInfo>>;

    async fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>>;
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

    async fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>> {
        if let ChainResponse::BlockOption(block) =
            self.send(ChainRequest::GetBlockByHash(hash)).await??
        {
            match block {
                Some(b) => Ok(Some(*b)),
                None => Ok(None),
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

    async fn get_block_info_by_hash(&self, hash: &HashValue) -> Result<Option<BlockInfo>> {
        if let ChainResponse::BlockInfoOption(block_info) =
            self.send(ChainRequest::GetBlockInfoByHash(*hash)).await??
        {
            return Ok(*block_info);
        }
        Ok(None)
    }

    async fn get_block_info_by_number(&self, number: u64) -> Result<Option<BlockInfo>> {
        if let ChainResponse::BlockInfoOption(block_info) = self
            .send(ChainRequest::GetBlockInfoByNumber(number))
            .await??
        {
            return Ok(*block_info);
        }
        Ok(None)
    }

    async fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<Transaction>> {
        let response = self.send(ChainRequest::GetTransaction(txn_hash)).await??;
        if let ChainResponse::TransactionOption(txn) = response {
            Ok(txn.map(|b| *b))
        } else {
            bail!("get transaction error.")
        }
    }

    async fn get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> Result<Option<RichTransactionInfo>> {
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

    async fn get_block_txn_infos(&self, block_hash: HashValue) -> Result<Vec<RichTransactionInfo>> {
        let response = self
            .send(ChainRequest::GetBlockTransactionInfos(block_hash))
            .await??;
        if let ChainResponse::TransactionInfos(txn_infos) = response {
            Ok(txn_infos)
        } else {
            bail!("get block's transaction_info error.")
        }
    }

    async fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<RichTransactionInfo>> {
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

    async fn main_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>> {
        if let ChainResponse::BlockOption(block) =
            self.send(ChainRequest::GetBlockByNumber(number)).await??
        {
            Ok(block.map(|b| *b))
        } else {
            bail!("Get chain block by number response error.")
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

    async fn main_block_header_by_number(
        &self,
        number: BlockNumber,
    ) -> Result<Option<BlockHeader>> {
        if let ChainResponse::BlockHeaderOption(header) = self
            .send(ChainRequest::GetBlockHeaderByNumber(number))
            .await??
        {
            return Ok(*header);
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

    async fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<RichTransactionInfo>> {
        let response = self
            .send(ChainRequest::GetTransactionInfos {
                start_index,
                reverse,
                max_size,
            })
            .await??;
        if let ChainResponse::TransactionInfos(tx_infos) = response {
            Ok(tx_infos)
        } else {
            bail!("get txn infos error")
        }
    }

    async fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>> {
        let response = self
            .send(ChainRequest::GetTransactionProof {
                block_id,
                transaction_global_index,
                event_index,
                access_path,
            })
            .await??;
        if let ChainResponse::TransactionProof(proof) = response {
            Ok(*proof)
        } else {
            bail!("get transactin proof error")
        }
    }
}
