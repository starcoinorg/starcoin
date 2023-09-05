use network_p2p_core::PeerId;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::Block;

#[derive(Clone, Debug, Hash, Eq, PartialOrd, Ord, PartialEq, Serialize, Deserialize)]
pub struct RelationshipPair {
    pub parent: HashValue,
    pub child: HashValue,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetDagAccumulatorLeaves {
    pub accumulator_leaf_index: u64,
    pub batch_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TargetDagAccumulatorLeaf {
    pub accumulator_root: HashValue, // accumulator info root
    pub leaf_index: u64,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct GetTargetDagAccumulatorLeafDetail {
    pub leaf_index: u64,
    pub batch_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct TargetDagAccumulatorLeafDetail {
    pub accumulator_root: HashValue,
    pub tips: Vec<HashValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetSyncDagBlockInfo {
    pub leaf_index: u64,
    pub batch_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncDagBlockInfo {
    pub block_id: HashValue,
    pub block: Option<Block>,
    pub peer_id: Option<PeerId>,
    pub dag_parents: Vec<HashValue>,
    pub dag_transaction_header: Option<HashValue>,
}
