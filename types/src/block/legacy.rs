use crate::block::BlockHeader;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};
use starcoin_crypto::HashValue;
use starcoin_uint::U256;
use starcoin_vm_types::transaction::SignedUserTransaction;

#[derive(
    Default, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash,
)]
pub struct BlockBody {
    /// The transactions in this block.
    pub transactions: Vec<SignedUserTransaction>,
    /// uncles block header
    pub uncles: Option<Vec<BlockHeader>>,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct Block {
    /// The header of this block.
    pub header: BlockHeader,
    /// The body of this block.
    pub body: BlockBody,
}

#[derive(
    Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash, JsonSchema,
)]
pub struct BlockInfo {
    /// Block id
    pub block_id: HashValue,
    /// The total difficulty.
    #[schemars(with = "String")]
    pub total_difficulty: U256,
    /// The transaction accumulator info
    pub txn_accumulator_info: AccumulatorInfo,
    /// The block accumulator info.
    pub block_accumulator_info: AccumulatorInfo,
}

impl From<BlockInfo> for super::BlockInfo {
    fn from(legacy_block_info: BlockInfo) -> Self {
        super::BlockInfo {
            block_id: legacy_block_info.block_id,
            total_difficulty: legacy_block_info.total_difficulty,
            txn_accumulator_info: legacy_block_info.txn_accumulator_info,
            block_accumulator_info: legacy_block_info.block_accumulator_info,
            vm_state_accumulator_info: AccumulatorInfo::default(),
        }
    }
}
