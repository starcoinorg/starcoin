// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;
use anyhow::Result;
use futures::future::BoxFuture;
use futures::FutureExt;
use network_p2p_core::NetRpcError;
use starcoin_accumulator::AccumulatorNode;
use starcoin_chain_service::{ChainAsyncService, ChainReaderService};
use starcoin_crypto::HashValue;
use starcoin_network_rpc_api::{
    dag_protocol, gen_server, BlockBody, GetAccountState, GetAccumulatorNodeByNodeHash,
    GetBlockHeadersByNumber, GetBlockIds, GetStateWithProof, GetStateWithTableItemProof,
    GetTxnsWithHash, GetTxnsWithSize, Ping, RpcRequest, MAX_BLOCK_HEADER_REQUEST_SIZE,
    MAX_BLOCK_INFO_REQUEST_SIZE, MAX_BLOCK_REQUEST_SIZE, MAX_TXN_REQUEST_SIZE,
};
use starcoin_service_registry::ServiceRef;
use starcoin_state_api::{ChainStateAsyncService, StateWithProof, StateWithTableItemProof};
use starcoin_state_service::ChainStateService;
use starcoin_state_tree::StateNode;
use starcoin_storage::Store;
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::block::Block;
use starcoin_types::{
    account_state::AccountState,
    block::{BlockHeader, BlockInfo, BlockNumber},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};
use std::sync::Arc;

pub struct NetworkRpcImpl {
    storage: Arc<dyn Store>,
    chain_service: ServiceRef<ChainReaderService>,
    txpool_service: TxPoolService,
    state_service: ServiceRef<ChainStateService>,
}

impl NetworkRpcImpl {
    pub fn new(
        storage: Arc<dyn Store>,
        chain_service: ServiceRef<ChainReaderService>,
        txpool: TxPoolService,
        state_service: ServiceRef<ChainStateService>,
    ) -> Self {
        Self {
            chain_service,
            txpool_service: txpool,
            storage,
            state_service,
        }
    }
}

impl gen_server::NetworkRpc for NetworkRpcImpl {
    fn get_txns_from_pool(
        &self,
        _peer_id: PeerId,
        req: GetTxnsWithSize,
    ) -> BoxFuture<Result<Vec<SignedUserTransaction>>> {
        let txpool = self.txpool_service.clone();
        let max_size = if req.max_size < MAX_TXN_REQUEST_SIZE {
            req.max_size
        } else {
            MAX_TXN_REQUEST_SIZE
        };
        let fut = async move { Ok(txpool.get_pending_txns(Some(max_size), None)) };
        Box::pin(fut)
    }

    fn get_txns_with_hash_from_pool(
        &self,
        _peer_id: PeerId,
        req: GetTxnsWithHash,
    ) -> BoxFuture<Result<Vec<Option<SignedUserTransaction>>>> {
        let txpool = self.txpool_service.clone();
        let fut = async move {
            let mut data = vec![];
            for id in req.ids {
                data.push(txpool.find_txn(&id));
            }
            Ok(data)
        };
        Box::pin(fut)
    }

    fn get_txns(
        &self,
        _peer_id: PeerId,
        req: GetTxnsWithHash,
    ) -> BoxFuture<Result<Vec<Option<Transaction>>>> {
        let storage = self.storage.clone();
        let fut = async move { storage.get_transactions(req.ids) };
        Box::pin(fut)
    }

    fn get_txn_infos(
        &self,
        _peer_id: PeerId,
        block_id: HashValue,
    ) -> BoxFuture<Result<Option<Vec<TransactionInfo>>>> {
        let storage = self.storage.clone();
        let fut = async move {
            if let Ok(txn_infos) = storage.get_block_transaction_infos(block_id) {
                Ok(Some(
                    txn_infos
                        .into_iter()
                        .map(|info| info.transaction_info)
                        .collect(),
                ))
            } else {
                Ok(None)
            }
        };
        Box::pin(fut)
    }

    fn get_headers_by_number(
        &self,
        _peer_id: PeerId,
        request: GetBlockHeadersByNumber,
    ) -> BoxFuture<Result<Vec<Option<BlockHeader>>>> {
        let chain_reader = self.chain_service.clone();

        let fut = async move {
            request.verify()?;
            let mut headers = Vec::new();
            let numbers: Vec<BlockNumber> = request.into();
            for number in numbers.into_iter() {
                let header = chain_reader
                    .clone()
                    .main_block_header_by_number(number)
                    .await?;
                headers.push(header);
            }
            Ok(headers)
        };
        Box::pin(fut)
    }

    fn get_headers_by_hash(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockHeader>>>> {
        let chain_reader = self.chain_service.clone();
        let fut = async move {
            if hashes.len() as u64 > MAX_BLOCK_HEADER_REQUEST_SIZE {
                return Err(NetRpcError::client_err(format!(
                    "max ids size > {}",
                    MAX_BLOCK_HEADER_REQUEST_SIZE
                ))
                .into());
            }
            chain_reader.clone().get_headers(hashes).await
        };
        Box::pin(fut)
    }

    fn get_block_infos(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockInfo>>>> {
        let chain_reader = self.chain_service.clone();
        let fut = async move {
            if hashes.len() as u64 > MAX_BLOCK_INFO_REQUEST_SIZE {
                return Err(NetRpcError::client_err(format!(
                    "max ids size > {}",
                    MAX_BLOCK_INFO_REQUEST_SIZE
                ))
                .into());
            }
            let infos = chain_reader.get_block_infos(hashes).await?;
            Ok(infos)
        };
        Box::pin(fut)
    }

    fn get_bodies_by_hash(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<BlockBody>>>> {
        let chain_reader = self.chain_service.clone();
        let fut = async move {
            if hashes.len() as u64 > MAX_BLOCK_REQUEST_SIZE {
                return Err(NetRpcError::client_err(format!(
                    "max ids size > {}",
                    MAX_BLOCK_REQUEST_SIZE
                ))
                .into());
            }
            let blocks = chain_reader.get_blocks(hashes).await?;
            let mut bodies = vec![];
            for block in blocks {
                bodies.push(block.map(|(block, _)| block.body));
            }
            Ok(bodies)
        };
        Box::pin(fut)
    }

    fn get_state_node_by_node_hash(
        &self,
        _peer_id: PeerId,
        state_node_key: HashValue,
    ) -> BoxFuture<Result<Option<StateNode>>> {
        let storage = self.storage.clone();
        let fut = async move { storage.get(&state_node_key) };
        Box::pin(fut)
    }

    fn get_accumulator_node_by_node_hash(
        &self,
        _peer_id: PeerId,
        request: GetAccumulatorNodeByNodeHash,
    ) -> BoxFuture<Result<Option<AccumulatorNode>>> {
        let storage = self.storage.clone();
        let acc_store = storage.get_accumulator_store(request.accumulator_storage_type);
        let fut = async move { acc_store.get_node(request.node_hash) };
        Box::pin(fut)
    }

    fn get_state_with_proof(
        &self,
        _peer_id: PeerId,
        req: GetStateWithProof,
    ) -> BoxFuture<Result<StateWithProof>> {
        let state_service = self.state_service.clone();
        let fut = async move {
            state_service
                .get_with_proof_by_root(req.access_path, req.state_root)
                .await
        };
        Box::pin(fut)
    }

    fn get_state_with_table_item_proof(
        &self,
        _peer_id: PeerId,
        req: GetStateWithTableItemProof,
    ) -> BoxFuture<Result<StateWithTableItemProof>> {
        let state_service = self.state_service.clone();
        let fut = async move {
            state_service
                .get_with_table_item_proof_by_root(req.handle, req.key, req.state_root)
                .await
        };
        Box::pin(fut)
    }

    fn get_account_state(
        &self,
        _peer_id: PeerId,
        req: GetAccountState,
    ) -> BoxFuture<Result<Option<AccountState>>> {
        let state_service = self.state_service.clone();
        let fut = async move {
            state_service
                .get_account_state_by_root(req.account_address, req.state_root)
                .await
        };
        Box::pin(fut)
    }

    fn ping(&self, _peer_id: PeerId, req: Ping) -> BoxFuture<Result<String>> {
        if req.err {
            futures::future::ready(Err(NetRpcError::client_err(req.msg).into())).boxed()
        } else {
            futures::future::ready(Ok(req.msg)).boxed()
        }
    }

    fn get_block_ids(
        &self,
        _peer_id: PeerId,
        req: GetBlockIds,
    ) -> BoxFuture<Result<Vec<HashValue>>> {
        let chain_service = self.chain_service.clone();
        let fut = async move {
            req.verify()?;
            chain_service
                .get_block_ids(req.start_number, req.reverse, req.max_size)
                .await
        };
        Box::pin(fut)
    }

    fn get_blocks(
        &self,
        _peer_id: PeerId,
        ids: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<Option<(Block, Option<Vec<HashValue>>)>>>> {
        let chain_service = self.chain_service.clone();
        let fut = async move {
            if ids.len() as u64 > MAX_BLOCK_REQUEST_SIZE {
                return Err(NetRpcError::client_err(format!(
                    "max block ids size > {}",
                    MAX_BLOCK_REQUEST_SIZE
                ))
                .into());
            }
            chain_service.get_blocks(ids).await
        };
        Box::pin(fut)
    }

    fn get_dag_accumulator_leaves(
        &self,
        _peer_id: PeerId,
        req: dag_protocol::GetDagAccumulatorLeaves,
    ) -> BoxFuture<Result<Vec<dag_protocol::TargetDagAccumulatorLeaf>>> {
        let chain_service = self.chain_service.clone();
        let fut = async move { chain_service.get_dag_accumulator_leaves(req).await };
        Box::pin(fut)
    }

    fn get_accumulator_leaf_detail(
        &self,
        _peer_id: PeerId,
        req: dag_protocol::GetTargetDagAccumulatorLeafDetail,
    ) -> BoxFuture<Result<Option<Vec<dag_protocol::TargetDagAccumulatorLeafDetail>>>> {
        let chain_service = self.chain_service.clone();
        let fut = async move { chain_service.get_dag_accumulator_leaves_detail(req).await };
        Box::pin(fut)
    }

    fn get_dag_block_info(
        &self,
        _peer_id: PeerId,
        _req: dag_protocol::GetSyncDagBlockInfo,
    ) -> BoxFuture<Result<Option<Vec<dag_protocol::SyncDagBlockInfo>>>> {
        todo!()
    }
}
