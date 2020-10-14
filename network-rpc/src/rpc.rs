// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use accumulator::AccumulatorNode;
use anyhow::Result;
use crypto::HashValue;
use futures::future::BoxFuture;
use futures::FutureExt;
use network_rpc_core::NetRpcError;
use starcoin_chain_service::ChainReaderService;
use starcoin_network_rpc_api::{
    gen_server, BlockBody, GetAccountState, GetAccumulatorNodeByNodeHash, GetBlockHeaders,
    GetBlockHeadersByNumber, GetStateWithProof, GetTxns, Ping, TransactionsData,
};
use starcoin_service_registry::ServiceRef;
use starcoin_state_api::{ChainStateAsyncService, StateWithProof};
use starcoin_state_service::ChainStateService;
use starcoin_storage::Store;
use starcoin_types::{
    account_state::AccountState,
    block::{BlockHeader, BlockInfo, BlockNumber},
    peer_info::PeerId,
    transaction::TransactionInfo,
};
use state_tree::StateNode;
use std::sync::Arc;
use traits::ChainAsyncService;
use txpool::TxPoolService;
use txpool_api::TxPoolSyncService;

//TODO Define a more suitable value and check
const MAX_REQUEST_SIZE: usize = 10;

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
        req: GetTxns,
    ) -> BoxFuture<Result<TransactionsData>> {
        let txpool = self.txpool_service.clone();
        let storage = self.storage.clone();
        let fut = async move {
            let data = {
                match req.ids {
                    // get from txpool
                    None => txpool.get_pending_txns(None, None),
                    // get from storage
                    Some(ids) => {
                        let mut data = vec![];
                        for id in ids {
                            if let Ok(txn) = storage.get_transaction(id) {
                                if let Some(txn) = txn {
                                    if let Ok(stxn) = txn.as_signed_user_txn() {
                                        data.push(stxn.clone());
                                    }
                                }
                            }
                        }
                        data
                    }
                }
            };
            Ok(TransactionsData { txns: data })
        };
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
                Ok(Some(txn_infos))
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
    ) -> BoxFuture<Result<Vec<BlockHeader>>> {
        let chain_reader = self.chain_service.clone();

        let fut = async move {
            let mut headers = Vec::new();
            let numbers: Vec<BlockNumber> = request.into();
            for number in numbers.into_iter() {
                if headers.len() >= MAX_REQUEST_SIZE {
                    break;
                }
                let header = chain_reader
                    .clone()
                    .master_block_header_by_number(number)
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
    ) -> BoxFuture<Result<Vec<BlockHeader>>> {
        let fut = async move {
            let mut headers = Vec::new();
            let chain_reader = self.chain_service.clone();
            for hash in hashes {
                if headers.len() >= MAX_REQUEST_SIZE {
                    break;
                }
                if let Ok(Some(block_header)) = chain_reader.clone().get_header_by_hash(&hash).await
                {
                    headers.push(block_header);
                }
            }
            Ok(headers)
        };
        Box::pin(fut)
    }

    fn get_headers(
        &self,
        _peer_id: PeerId,
        request: GetBlockHeaders,
    ) -> BoxFuture<Result<Vec<BlockHeader>>> {
        let chain_reader = self.chain_service.clone();
        let fut = async move {
            let mut headers = Vec::new();
            if let Ok(Some(header)) = chain_reader
                .clone()
                .get_header_by_hash(&request.block_id)
                .await
            {
                let numbers: Vec<BlockNumber> = request.into_numbers(header.number());
                for number in numbers.into_iter() {
                    if headers.len() >= MAX_REQUEST_SIZE {
                        break;
                    }
                    if let Ok(header) = chain_reader
                        .clone()
                        .master_block_header_by_number(number)
                        .await
                    {
                        headers.push(header);
                    }
                }
            }
            Ok(headers)
        };
        Box::pin(fut)
    }

    fn get_block_infos(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>> {
        let chain_reader = self.chain_service.clone();
        let fut = async move {
            let mut infos = Vec::new();
            for hash in hashes {
                if infos.len() >= MAX_REQUEST_SIZE {
                    break;
                }
                if let Ok(Some(block_info)) = chain_reader.get_block_info_by_hash(&hash).await {
                    infos.push(block_info);
                }
            }
            Ok(infos)
        };
        Box::pin(fut)
    }

    fn get_bodies_by_hash(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockBody>>> {
        let chain_reader = self.chain_service.clone();
        let fut = async move {
            let mut bodies = Vec::new();
            for hash in hashes {
                if bodies.len() >= MAX_REQUEST_SIZE {
                    break;
                }
                let (transactions, uncles) = match chain_reader.get_block_by_hash(hash).await {
                    Ok(block) => (
                        block.transactions().to_vec(),
                        if block.uncles().is_some() {
                            Some(block.uncles().expect("block.uncles() is none.").to_vec())
                        } else {
                            None
                        },
                    ),
                    _ => (Vec::new(), None),
                };

                let body = BlockBody {
                    transactions,
                    hash,
                    uncles,
                };
                bodies.push(body);
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
}
