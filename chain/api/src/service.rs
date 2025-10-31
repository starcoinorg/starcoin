// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::message::{ChainRequest, ChainResponse};
use crate::TransactionInfoWithProof;
use anyhow::{bail, Result};
use starcoin_crypto::HashValue;
use starcoin_service_registry::{ActorService, ServiceHandler, ServiceRef};
use starcoin_types::contract_event::{ContractEvent, StcContractEvent, StcContractEventInfo};
use starcoin_types::filter::Filter;
use starcoin_types::multi_access_path::MultiAccessPath;
use starcoin_types::multi_state::MultiState;
use starcoin_types::startup_info::ChainStatus;
use starcoin_types::transaction::{StcRichTransactionInfo, StcTransaction};
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    startup_info::StartupInfo,
};
use starcoin_vm2_vm_types::access_path::AccessPath as AccessPath2;

/// Readable block chain service trait
pub trait ReadableChainService {
    fn get_header_by_hash(&self, hash: HashValue) -> Result<Option<BlockHeader>>;
    fn get_block_by_hash(&self, hash: HashValue) -> Result<Option<Block>>;
    fn get_blocks(&self, ids: Vec<HashValue>) -> Result<Vec<Option<Block>>>;
    fn get_headers(&self, ids: Vec<HashValue>) -> Result<Vec<Option<BlockHeader>>>;
    fn get_block_info_by_hash(&self, hash: HashValue) -> Result<Option<BlockInfo>>;
    fn get_transaction(&self, hash: HashValue) -> Result<Option<StcTransaction>>;
    fn get_transaction_info(&self, txn_hash: HashValue) -> Result<Option<StcRichTransactionInfo>>;
    fn get_block_txn_infos(&self, block_id: HashValue) -> Result<Vec<StcRichTransactionInfo>>;
    fn get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<StcRichTransactionInfo>>;
    fn get_events_by_txn_info_hash(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<ContractEvent>>>;

    fn get_events_by_txn_info_hash2(
        &self,
        txn_info_id: HashValue,
    ) -> Result<Option<Vec<StcContractEvent>>>;

    /// for main
    fn main_head_header(&self) -> BlockHeader;
    fn main_head_block(&self) -> Block;
    fn main_block_by_number(&self, number: BlockNumber) -> Result<Option<Block>>;
    fn main_block_header_by_number(&self, number: BlockNumber) -> Result<Option<BlockHeader>>;
    fn main_block_info_by_number(&self, number: BlockNumber) -> Result<Option<BlockInfo>>;
    fn main_startup_info(&self) -> StartupInfo;
    fn main_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        reverse: bool,
        count: u64,
    ) -> Result<Vec<Block>>;
    fn get_main_events(&self, filter: Filter) -> Result<Vec<StcContractEventInfo>>;
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
    ) -> Result<Vec<StcRichTransactionInfo>>;

    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> Result<Option<TransactionInfoWithProof>>;

    fn get_block_infos(&self, ids: Vec<HashValue>) -> Result<Vec<Option<BlockInfo>>>;

    fn get_dag_block_children(&self, ids: Vec<HashValue>) -> Result<Vec<HashValue>>;
    fn get_dag_state(&self) -> Result<starcoin_dag::consensusdb::consensus_state::DagStateView>;
    fn get_ghostdagdata(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<starcoin_dag::types::ghostdata::GhostdagData>>>;

    fn get_range_in_location(
        &self,
        start_id: HashValue,
        end_id: Option<HashValue>,
    ) -> Result<crate::range_locate::RangeInLocation>;

    fn get_absent_blocks(&self, absent_id: Vec<HashValue>, exp: u64) -> Result<Vec<Block>>;
}

/// Writeable block chain service trait
pub trait WriteableChainService: Send + Sync {
    fn try_connect(&mut self, block: Block) -> Result<()>;
}

pub trait ChainAsyncService:
    Clone + std::marker::Unpin + std::marker::Sync + std::marker::Send
{
    fn get_header_by_hash(
        &self,
        hash: &HashValue,
    ) -> impl std::future::Future<Output = Result<Option<BlockHeader>>> + Send;
    fn get_block_by_hash(
        &self,
        hash: HashValue,
    ) -> impl std::future::Future<Output = Result<Option<Block>>> + Send;
    fn get_blocks(
        &self,
        hashes: Vec<HashValue>,
    ) -> impl std::future::Future<Output = Result<Vec<Option<Block>>>> + Send;
    fn get_headers(
        &self,
        hashes: Vec<HashValue>,
    ) -> impl std::future::Future<Output = Result<Vec<Option<BlockHeader>>>> + Send;
    fn get_block_info_by_hash(
        &self,
        hash: &HashValue,
    ) -> impl std::future::Future<Output = Result<Option<BlockInfo>>> + Send;
    fn get_block_info_by_number(
        &self,
        number: u64,
    ) -> impl std::future::Future<Output = Result<Option<BlockInfo>>> + Send;
    fn get_transaction(
        &self,
        txn_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<Option<StcTransaction>>> + Send;
    fn get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<Option<StcRichTransactionInfo>>> + Send;
    fn get_transaction_block(
        &self,
        txn_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<Option<Block>>> + Send;
    fn get_block_txn_infos(
        &self,
        block_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<Vec<StcRichTransactionInfo>>> + Send;
    fn get_txn_info_by_block_and_index(
        &self,
        block_hash: HashValue,
        idx: u64,
    ) -> impl std::future::Future<Output = Result<Option<StcRichTransactionInfo>>> + Send;
    fn get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<Vec<StcContractEventInfo>>> + Send;
    fn get_events_by_txn_hash2(
        &self,
        txn_hash: HashValue,
    ) -> impl std::future::Future<Output = Result<Vec<StcContractEventInfo>>> + Send;

    /// for main
    fn main_head_header(&self) -> impl std::future::Future<Output = Result<BlockHeader>> + Send;
    fn main_head_block(&self) -> impl std::future::Future<Output = Result<Block>> + Send;
    fn main_block_by_number(
        &self,
        number: BlockNumber,
    ) -> impl std::future::Future<Output = Result<Option<Block>>> + Send;
    fn main_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        reverse: bool,
        count: u64,
    ) -> impl std::future::Future<Output = Result<Vec<Block>>> + Send;
    fn main_block_header_by_number(
        &self,
        number: BlockNumber,
    ) -> impl std::future::Future<Output = Result<Option<BlockHeader>>> + Send;
    fn main_startup_info(&self) -> impl std::future::Future<Output = Result<StartupInfo>> + Send;
    fn main_status(&self) -> impl std::future::Future<Output = Result<ChainStatus>> + Send;
    fn main_events(
        &self,
        filter: Filter,
    ) -> impl std::future::Future<Output = Result<Vec<StcContractEventInfo>>> + Send;
    fn get_block_ids(
        &self,
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    ) -> impl std::future::Future<Output = Result<Vec<HashValue>>> + Send;
    fn get_transaction_infos(
        &self,
        start_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> impl std::future::Future<Output = Result<Vec<StcRichTransactionInfo>>> + Send;

    fn get_transaction_proof(
        &self,
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> impl std::future::Future<Output = Result<Option<TransactionInfoWithProof>>> + Send;

    fn get_block_infos(
        &self,
        hashes: Vec<HashValue>,
    ) -> impl std::future::Future<Output = Result<Vec<Option<BlockInfo>>>> + Send;
    fn get_multi_state_by_hash(
        &self,
        hash: HashValue,
    ) -> impl std::future::Future<Output = Result<Option<MultiState>>> + Send;

    fn get_dag_block_children(
        &self,
        hashes: Vec<HashValue>,
    ) -> impl std::future::Future<Output = Result<Vec<HashValue>>> + Send;

    fn get_ghostdagdata(
        &self,
        ids: Vec<HashValue>,
    ) -> impl std::future::Future<
        Output = Result<Vec<Option<starcoin_dag::types::ghostdata::GhostdagData>>>,
    > + Send;

    fn get_range_in_location(
        &self,
        req: starcoin_network_rpc_api::GetRangeInLocationRequest,
    ) -> impl std::future::Future<
        Output = Result<starcoin_network_rpc_api::GetRangeInLocationResponse>,
    > + Send;

    fn get_absent_blocks(
        &self,
        req: starcoin_network_rpc_api::GetAbsentBlockRequest,
    ) -> impl std::future::Future<Output = Result<starcoin_network_rpc_api::GetAbsentBlockResponse>> + Send;
}

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

    async fn get_headers(&self, ids: Vec<HashValue>) -> Result<Vec<Option<BlockHeader>>> {
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

    async fn get_transaction(&self, txn_hash: HashValue) -> Result<Option<StcTransaction>> {
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
    ) -> Result<Option<StcRichTransactionInfo>> {
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

    async fn get_block_txn_infos(
        &self,
        block_hash: HashValue,
    ) -> Result<Vec<StcRichTransactionInfo>> {
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
    ) -> Result<Option<StcRichTransactionInfo>> {
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
    async fn get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<StcContractEventInfo>> {
        let response = self
            .send(ChainRequest::GetEventsByTxnHash { txn_hash })
            .await??;
        if let ChainResponse::Events(events) = response {
            Ok(events)
        } else {
            bail!("get txn info by block and idx error.")
        }
    }

    async fn get_events_by_txn_hash2(
        &self,
        txn_hash: HashValue,
    ) -> Result<Vec<StcContractEventInfo>> {
        let response = self
            .send(ChainRequest::GetEventsByTxnHash2 { txn_hash })
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
        reverse: bool,
        count: u64,
    ) -> Result<Vec<Block>> {
        if let ChainResponse::BlockVec(blocks) = self
            .send(ChainRequest::GetBlocksByNumber(number, reverse, count))
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

    async fn main_events(&self, filter: Filter) -> Result<Vec<StcContractEventInfo>> {
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
    ) -> Result<Vec<StcRichTransactionInfo>> {
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
        access_path: Option<MultiAccessPath>,
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
            bail!("get transaction proof error")
        }
    }

    async fn get_block_infos(&self, hashes: Vec<HashValue>) -> Result<Vec<Option<BlockInfo>>> {
        let response = self.send(ChainRequest::GetBlockInfos(hashes)).await??;
        if let ChainResponse::BlockInfoVec(block_infos) = response {
            Ok(*block_infos)
        } else {
            bail!("get block_infos error")
        }
    }

    async fn get_multi_state_by_hash(&self, hash: HashValue) -> Result<Option<MultiState>> {
        let response = self.send(ChainRequest::GetMultiStateByHash(hash)).await??;
        if let ChainResponse::MultiStateResp(multi_state) = response {
            Ok(multi_state)
        } else {
            bail!("get multi state error")
        }
    }

    async fn get_dag_block_children(&self, block_ids: Vec<HashValue>) -> Result<Vec<HashValue>> {
        let response = self
            .send(ChainRequest::GetDagBlockChildren { block_ids })
            .await??;
        if let ChainResponse::HashVec(hashes) = response {
            Ok(hashes)
        } else {
            bail!("get dag block children error")
        }
    }

    async fn get_ghostdagdata(
        &self,
        ids: Vec<HashValue>,
    ) -> Result<Vec<Option<starcoin_dag::types::ghostdata::GhostdagData>>> {
        let response = self.send(ChainRequest::GetGhostdagData(ids)).await??;
        if let ChainResponse::GhostdagDataOption(ghostdag_data) = response {
            Ok(*ghostdag_data)
        } else {
            bail!("failed to get ghostdag data")
        }
    }

    async fn get_range_in_location(
        &self,
        req: starcoin_network_rpc_api::GetRangeInLocationRequest,
    ) -> Result<starcoin_network_rpc_api::GetRangeInLocationResponse> {
        let response = self
            .send(ChainRequest::GetRangeInLocation {
                start_id: req.start_id,
                end_id: req.end_id,
            })
            .await??;
        if let ChainResponse::GetRangeInLocation { range } = response {
            Ok(starcoin_network_rpc_api::GetRangeInLocationResponse {
                range: match range {
                    crate::range_locate::RangeInLocation::NotInSelectedChain => {
                        starcoin_network_rpc_api::RangeInLocation::NotInSelectedChain
                    }
                    crate::range_locate::RangeInLocation::InSelectedChain(hash, hashes) => {
                        starcoin_network_rpc_api::RangeInLocation::InSelectedChain(hash, hashes)
                    }
                },
            })
        } else {
            bail!("get range in location error")
        }
    }

    async fn get_absent_blocks(
        &self,
        req: starcoin_network_rpc_api::GetAbsentBlockRequest,
    ) -> Result<starcoin_network_rpc_api::GetAbsentBlockResponse> {
        let response = self
            .send(ChainRequest::GetAbsentBlocks {
                absent_id: req.absent_id,
                exp: req.exp,
            })
            .await??;
        if let ChainResponse::GetAbsentBlocks { absent_blocks } = response {
            Ok(starcoin_network_rpc_api::GetAbsentBlockResponse { absent_blocks })
        } else {
            bail!("get absent blocks error")
        }
    }
}
