use accumulator::AccumulatorNode;

use futures::future::BoxFuture;
use futures::FutureExt;
use network_api::{messages::PeerMessage, NetworkService, PeerId, PeerProvider};
use network_rpc_core::RawRpcClient;
use network_rpc_core::Result;
use starcoin_chain::BlockChain;
use starcoin_crypto::HashValue;
use starcoin_network_rpc_api::{
    BlockBody, GetAccumulatorNodeByNodeHash, GetBlockHeaders, GetBlockHeadersByNumber,
};
use starcoin_traits::ChainReader;
use starcoin_types::block::{BlockHeader, BlockInfo, BlockNumber};
use starcoin_types::peer_info::{PeerInfo, RpcInfo};
use starcoin_types::system_events::NewHeadBlock;
use starcoin_types::transaction::TransactionInfo;
use state_tree::StateNode;
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;

const MAX_SIZE: usize = 10;

#[derive(Clone)]
pub struct DummyNetworkService {
    peer_id: PeerId,
    peers: Vec<PeerInfo>,
    chain: Arc<BlockChain>,
}

impl DummyNetworkService {
    pub fn new(chain: Arc<BlockChain>) -> Self {
        let mut peers: Vec<PeerInfo> = Vec::new();
        peers.push(PeerInfo::random());
        peers.push(PeerInfo::random());
        Self {
            peer_id: PeerId::random(),
            peers,
            chain,
        }
    }

    fn get_headers_by_number(&self, request: GetBlockHeadersByNumber) -> Result<Vec<BlockHeader>> {
        let mut headers = Vec::new();
        let numbers: Vec<BlockNumber> = request.into();
        for number in numbers.into_iter() {
            if headers.len() >= MAX_SIZE {
                break;
            }
            if let Ok(Some(header)) = self.chain.get_header_by_number(number) {
                headers.push(header);
            }
        }
        Ok(headers)
    }

    fn get_header_by_hash(&self, hashes: Vec<HashValue>) -> Result<Vec<BlockHeader>> {
        let mut headers = Vec::new();
        for hash in hashes {
            if headers.len() >= MAX_SIZE {
                break;
            }
            if let Ok(Some(block_header)) = self.chain.get_header(hash) {
                headers.push(block_header);
            }
        }
        Ok(headers)
    }

    fn get_headers_with_peer(&self, request: GetBlockHeaders) -> Result<Vec<BlockHeader>> {
        let mut headers = Vec::new();
        if let Ok(Some(header)) = self.chain.get_header(request.block_id) {
            let numbers: Vec<BlockNumber> = request.into_numbers(header.number());
            for number in numbers.into_iter() {
                if headers.len() >= MAX_SIZE {
                    break;
                }
                if let Ok(Some(header)) = self.chain.get_header_by_number(number) {
                    headers.push(header);
                }
            }
        }
        Ok(headers)
    }

    fn get_info_by_hash(&self, hashes: Vec<HashValue>) -> Result<Vec<BlockInfo>> {
        let mut infos = Vec::new();
        for hash in hashes {
            if infos.len() >= MAX_SIZE {
                break;
            }
            if let Ok(Some(block_info)) = self.chain.get_block_info(Some(hash)) {
                infos.push(block_info);
            }
        }
        Ok(infos)
    }

    fn get_body_by_hash(&self, hashes: Vec<HashValue>) -> Result<Vec<BlockBody>> {
        let mut bodies = Vec::new();
        for hash in hashes {
            if bodies.len() >= MAX_SIZE {
                break;
            }
            let (transactions, uncles) = match self.chain.get_block(hash) {
                Ok(Some(block)) => (
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
    }

    fn get_state_node_by_node_hash(&self, state_node_key: HashValue) -> Result<Option<StateNode>> {
        self.chain
            .get_storage()
            .get(&state_node_key)
            .map_err(|e| e.into())
    }

    fn get_accumulator_node_by_node_hash(
        &self,
        req: GetAccumulatorNodeByNodeHash,
    ) -> Result<Option<AccumulatorNode>> {
        self.chain
            .get_storage()
            .get_node(req.accumulator_storage_type, req.node_hash)
            .map_err(|e| e.into())
    }

    fn get_txn_infos(&self, block_id: HashValue) -> Result<Option<Vec<TransactionInfo>>> {
        if let Ok(txn_infos) = self
            .chain
            .get_storage()
            .get_block_transaction_infos(block_id)
        {
            Ok(Some(txn_infos))
        } else {
            Ok(None)
        }
    }

    async fn handle_request(
        &self,
        _peer_id: Option<PeerId>,
        rpc_path: String,
        message: Vec<u8>,
        _time_out: Duration,
    ) -> Result<Vec<u8>> {
        //TODO refactor this, do not use string to match method.
        match rpc_path.to_lowercase().as_str() {
            // "get_txns" => {}
            "get_txn_infos" => {
                let block_id: HashValue = scs::from_bytes(message.as_slice())?;
                let resp = self.get_txn_infos(block_id)?;
                Ok(scs::to_bytes(&resp)?)
            }
            "get_headers_by_number" => {
                let req: GetBlockHeadersByNumber = scs::from_bytes(message.as_slice())?;
                let resp = self.get_headers_by_number(req)?;
                Ok(scs::to_bytes(&resp)?)
            }
            "get_headers_by_hash" => {
                let req: Vec<HashValue> = scs::from_bytes(message.as_slice())?;
                let resp = self.get_header_by_hash(req)?;
                Ok(scs::to_bytes(&resp)?)
            }
            "get_headers" => {
                let req: GetBlockHeaders = scs::from_bytes(message.as_slice())?;
                let resp = self.get_headers_with_peer(req)?;
                Ok(scs::to_bytes(&resp)?)
            }
            "get_block_infos" => {
                let req: Vec<HashValue> = scs::from_bytes(message.as_slice())?;
                let resp = self.get_info_by_hash(req)?;
                Ok(scs::to_bytes(&resp)?)
            }
            "get_bodies_by_hash" => {
                let req: Vec<HashValue> = scs::from_bytes(message.as_slice())?;
                let resp = self.get_body_by_hash(req)?;
                Ok(scs::to_bytes(&resp)?)
            }
            "get_state_node_by_node_hash" => {
                let req: HashValue = scs::from_bytes(message.as_slice())?;
                let resp = self.get_state_node_by_node_hash(req)?;
                Ok(scs::to_bytes(&resp)?)
            }
            "get_accumulator_node_by_node_hash" => {
                let req: GetAccumulatorNodeByNodeHash = scs::from_bytes(message.as_slice())?;
                let resp = self.get_accumulator_node_by_node_hash(req)?;
                Ok(scs::to_bytes(&resp)?)
            }
            // "get_state_with_proof" => {
            // }
            // "get_account_state" => {
            // }
            _ => unimplemented!(),
        }
    }

    async fn send_request_bytes(
        &self,
        peer_id: Option<PeerId>,
        rpc_path: String,
        message: Vec<u8>,
        time_out: Duration,
    ) -> anyhow::Result<Vec<u8>> {
        self.handle_request(peer_id, rpc_path, message, time_out)
            .then(|result| async move { Ok(scs::to_bytes(&result).unwrap()) })
            .await
    }
}

#[async_trait::async_trait]
impl NetworkService for DummyNetworkService {
    async fn send_peer_message(
        &self,
        _protocol_name: Cow<'static, [u8]>,
        _peer_id: PeerId,
        _msg: PeerMessage,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn broadcast_new_head_block(
        &self,
        _protocol_name: Cow<'static, [u8]>,
        _event: NewHeadBlock,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn register_rpc_proto(&self, _rpc_info: RpcInfo) -> anyhow::Result<()> {
        Ok(())
    }
}

impl RawRpcClient for DummyNetworkService {
    fn send_raw_request(
        &self,
        peer_id: Option<PeerId>,
        rpc_path: String,
        message: Vec<u8>,
        timeout: Duration,
    ) -> BoxFuture<anyhow::Result<Vec<u8>>> {
        self.send_request_bytes(peer_id, rpc_path, message, timeout)
            .boxed()
    }
}

impl PeerProvider for DummyNetworkService {
    fn identify(&self) -> PeerId {
        self.peer_id.clone()
    }

    fn peer_set(&self) -> BoxFuture<anyhow::Result<Vec<PeerInfo>>> {
        futures::future::ready(Ok(self.peers.clone())).boxed()
    }

    fn get_peer(&self, _peer_id: PeerId) -> BoxFuture<anyhow::Result<Option<PeerInfo>>> {
        unimplemented!()
    }
}
