use anyhow::{format_err, Result};
use crypto::hash::HashValue;
use crypto::hash::PlainCryptoHash;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_network_rpc_api::{
    gen_client::NetworkRpcClient, GetAccumulatorNodeByNodeHash, GetBlockHeaders,
    GetBlockHeadersByNumber, GetTxns, TransactionsData,
};
use starcoin_state_tree::StateNode;
use starcoin_types::{
    block::{BlockHeader, BlockInfo, BlockNumber},
    peer_info::PeerId,
    transaction::TransactionInfo,
};
use std::collections::HashSet;
use std::hash::Hash;

const HEAD_CT: usize = 10;
//TODO find a suitable value and strategy.
#[allow(dead_code)]
const STABLELIZE_BLCOK_NUM: usize = 7;

trait RpcVerify<C: Clone> {
    fn filter<T, F>(&mut self, rpc_data: Vec<T>, hash_fun: F) -> Vec<T>
    where
        F: Fn(&T) -> C;
}

struct RpcEntryVerify<C: Clone + Eq + Hash> {
    entries: HashSet<C>,
}

impl<C: Clone + Eq + Hash> RpcVerify<C> for RpcEntryVerify<C> {
    fn filter<T, F>(&mut self, rpc_data: Vec<T>, hash_fun: F) -> Vec<T>
    where
        F: Fn(&T) -> C,
    {
        let mut rpc_data = rpc_data;
        let mut dirty_data = Vec::new();
        for data in rpc_data.as_slice().iter() {
            let hash = hash_fun(data);
            if !self.entries.contains(&hash) {
                dirty_data.push(hash);
            }
        }

        if !dirty_data.is_empty() {
            for hash in dirty_data.as_slice().iter() {
                rpc_data.retain(|d| hash_fun(d) != *hash);
            }
        }

        rpc_data
    }
}

impl From<&Vec<HashValue>> for RpcEntryVerify<HashValue> {
    fn from(data: &Vec<HashValue>) -> Self {
        let mut entries = HashSet::new();
        for hash in data.iter() {
            entries.insert(*hash);
        }
        Self { entries }
    }
}

impl From<&Vec<BlockNumber>> for RpcEntryVerify<BlockNumber> {
    fn from(data: &Vec<BlockNumber>) -> Self {
        let mut entries = HashSet::new();
        for number in data.iter() {
            entries.insert(*number);
        }
        Self { entries }
    }
}

impl From<&GetTxns> for RpcEntryVerify<HashValue> {
    fn from(data: &GetTxns) -> Self {
        let mut entries = HashSet::new();
        if let Some(ids) = data.clone().ids {
            for hash in ids.into_iter() {
                entries.insert(hash);
            }
        }
        Self { entries }
    }
}

impl From<&GetBlockHeadersByNumber> for RpcEntryVerify<BlockNumber> {
    fn from(data: &GetBlockHeadersByNumber) -> Self {
        let numbers: Vec<BlockNumber> = data.clone().into();
        let mut entries = HashSet::new();
        for number in numbers.into_iter() {
            entries.insert(number);
        }
        Self { entries }
    }
}

pub async fn get_txns(
    client: &NetworkRpcClient,
    peer_id: PeerId,
    req: GetTxns,
) -> Result<TransactionsData> {
    let data = client.get_txns_from_pool(peer_id, req.clone()).await?;
    if req.ids.is_some() {
        let mut verify_condition: RpcEntryVerify<HashValue> = (&req).into();
        let verified_txns = verify_condition
            .filter((*data.get_txns()).to_vec(), |txn| -> HashValue {
                txn.crypto_hash()
            });
        Ok(TransactionsData {
            txns: verified_txns,
        })
    } else {
        Ok(data)
    }
}

pub async fn get_txn_infos(
    client: &NetworkRpcClient,
    peer_id: PeerId,
    block_id: HashValue,
) -> Result<Option<Vec<TransactionInfo>>> {
    client.get_txn_infos(peer_id, block_id).await
}

pub async fn get_headers_by_number(
    client: &NetworkRpcClient,
    peer_id: PeerId,
    req: GetBlockHeadersByNumber,
) -> Result<Vec<BlockHeader>> {
    let mut verify_condition: RpcEntryVerify<BlockNumber> = (&req).into();
    let data = client.get_headers_by_number(peer_id, req).await?;
    let verified_headers =
        verify_condition.filter(data, |header| -> BlockNumber { header.number() });
    Ok(verified_headers)
}

pub async fn get_headers_with_peer(
    client: &NetworkRpcClient,
    peer_id: PeerId,
    req: GetBlockHeaders,
    number: BlockNumber,
) -> Result<Vec<BlockHeader>> {
    let mut verify_condition: RpcEntryVerify<BlockNumber> =
        (&req.clone().into_numbers(number)).into();
    let data = client.get_headers(peer_id, req).await?;
    let verified_headers =
        verify_condition.filter(data, |header| -> BlockNumber { header.number() });
    Ok(verified_headers)
}

pub async fn get_block_infos(
    client: &NetworkRpcClient,
    peer_id: PeerId,
    hashes: Vec<HashValue>,
) -> Result<Vec<BlockInfo>> {
    let mut verify_condition: RpcEntryVerify<HashValue> = (&hashes).into();
    let data = client.get_block_infos(peer_id, hashes).await?;
    let verified_infos = verify_condition.filter(data, |info| -> HashValue { *info.block_id() });
    Ok(verified_infos)
}

pub async fn get_state_node_by_node_hash(
    client: &NetworkRpcClient,
    peer_id: PeerId,
    node_key: HashValue,
) -> Result<StateNode> {
    if let Some(state_node) = client
        .get_state_node_by_node_hash(peer_id, node_key)
        .await?
    {
        let state_node_id = state_node.inner().hash();
        if node_key == state_node_id {
            Ok(state_node)
        } else {
            Err(format_err!(
                "State node hash {:?} and node key {:?} mismatch.",
                state_node_id,
                node_key
            ))
        }
    } else {
        Err(format_err!(
            "State node is none by node key {:?}.",
            node_key
        ))
    }
}

pub async fn get_accumulator_node_by_node_hash(
    client: &NetworkRpcClient,
    peer_id: PeerId,
    node_key: HashValue,
    accumulator_type: AccumulatorStoreType,
) -> Result<AccumulatorNode> {
    if let Some(accumulator_node) = client
        .get_accumulator_node_by_node_hash(
            peer_id,
            GetAccumulatorNodeByNodeHash {
                node_hash: node_key,
                accumulator_storage_type: accumulator_type,
            },
        )
        .await?
    {
        let accumulator_node_id = accumulator_node.hash();
        if node_key == accumulator_node_id {
            Ok(accumulator_node)
        } else {
            Err(format_err!(
                "Accumulator node hash {:?} and node key {:?} mismatch.",
                accumulator_node_id,
                node_key
            ))
        }
    } else {
        Err(format_err!(
            "Accumulator node is none by node key {:?}.",
            node_key
        ))
    }
}

/// for common
pub fn get_headers_msg_for_common(block_id: HashValue) -> GetBlockHeaders {
    GetBlockHeaders::new(block_id, 1, false, HEAD_CT)
}

pub fn get_headers_msg_for_ancestor(
    block_number: BlockNumber,
    step: usize,
) -> GetBlockHeadersByNumber {
    //todo：binary search
    GetBlockHeadersByNumber::new(block_number, step, HEAD_CT)
}
