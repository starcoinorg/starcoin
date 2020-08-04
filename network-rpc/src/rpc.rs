// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use accumulator::AccumulatorNode;
use anyhow::*;
use chain::ChainActorRef;
use crypto::HashValue;
use futures::future::BoxFuture;
use starcoin_network_rpc_api::{
    gen_server, BlockBody, GetAccountState, GetAccumulatorNodeByNodeHash, GetBlockHeaders,
    GetBlockHeadersByNumber, GetStateWithProof, GetTxns, TransactionsData,
};
use state_api::{ChainStateAsyncService, StateWithProof};
use state_tree::StateNode;
use std::sync::Arc;
use storage::Store;
use traits::ChainAsyncService;
use txpool::TxPoolService;
use txpool_api::TxPoolSyncService;
use types::{
    account_state::AccountState,
    block::{BlockHeader, BlockInfo, BlockNumber},
    peer_info::PeerId,
    transaction::TransactionInfo,
};

const MAX_SIZE: usize = 10;

pub struct NetworkRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    chain_reader: ChainActorRef,
    txpool: TxPoolService,
    storage: Arc<dyn Store>,
    state_service: S,
}

impl<S> NetworkRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    pub fn new(
        chain_reader: ChainActorRef,
        txpool: TxPoolService,
        state_service: S,
        storage: Arc<dyn Store>,
    ) -> Self {
        Self {
            chain_reader,
            txpool,
            storage,
            state_service,
        }
    }
}

impl<S> gen_server::NetworkRpc for NetworkRpcImpl<S>
where
    S: ChainStateAsyncService + 'static,
{
    fn get_txns(&self, _peer_id: PeerId, req: GetTxns) -> BoxFuture<Result<TransactionsData>> {
        let txpool = self.txpool.clone();
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
        let chain_reader = self.chain_reader.clone();
        let fut = async move {
            let mut headers = Vec::new();
            let numbers: Vec<BlockNumber> = request.into();
            for number in numbers.into_iter() {
                if headers.len() >= MAX_SIZE {
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
            Ok(headers)
        };
        Box::pin(fut)
    }

    fn get_header_by_hash(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockHeader>>> {
        let fut = async move {
            let mut headers = Vec::new();
            let chain_reader = self.chain_reader.clone();
            for hash in hashes {
                if headers.len() >= MAX_SIZE {
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

    fn get_headers_with_peer(
        &self,
        _peer_id: PeerId,
        request: GetBlockHeaders,
    ) -> BoxFuture<Result<Vec<BlockHeader>>> {
        let chain_reader = self.chain_reader.clone();
        let fut = async move {
            let mut headers = Vec::new();
            if let Ok(Some(header)) = chain_reader
                .clone()
                .get_header_by_hash(&request.block_id)
                .await
            {
                let numbers: Vec<BlockNumber> = request.into_numbers(header.number());
                for number in numbers.into_iter() {
                    if headers.len() >= MAX_SIZE {
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

    fn get_info_by_hash(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>> {
        let fut = async move {
            let mut infos = Vec::new();
            let chain_reader = self.chain_reader.clone();
            for hash in hashes {
                if infos.len() >= MAX_SIZE {
                    break;
                }
                if let Ok(Some(block_info)) =
                    chain_reader.clone().get_block_info_by_hash(&hash).await
                {
                    infos.push(block_info);
                }
            }
            Ok(infos)
        };
        Box::pin(fut)
    }

    fn get_body_by_hash(
        &self,
        _peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockBody>>> {
        let chain_reader = self.chain_reader.clone();
        let fut = async move {
            let mut bodies = Vec::new();
            for hash in hashes {
                if bodies.len() >= MAX_SIZE {
                    break;
                }
                let (transactions, uncles) =
                    match chain_reader.clone().get_block_by_hash(hash).await {
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
        let fut =
            async move { storage.get_node(request.accumulator_storage_type, request.node_hash) };
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
}
