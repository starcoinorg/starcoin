use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_vm1_types::block::BlockInfo;
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_types::{
    view::{AccumulatorInfoView, StrView},
    U256,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlockInfoView2 {
    /// Block hash
    pub block_hash: HashValue,
    /// The total difficulty.
    #[schemars(with = "String")]
    pub total_difficulty: U256,
    /// The transaction accumulator info
    pub txn_accumulator_info: AccumulatorInfoView,
    /// The block accumulator info.
    pub block_accumulator_info: AccumulatorInfoView,
    /// The block accumulator info.
    pub vm_state_accumulator_info: AccumulatorInfoView,
}

fn info1_to_infoview2(info: &AccumulatorInfo) -> AccumulatorInfoView {
    AccumulatorInfoView {
        accumulator_root: info.accumulator_root,
        frozen_subtree_roots: info.frozen_subtree_roots.clone(),
        num_leaves: StrView::from(info.num_leaves),
        num_nodes: StrView::from(info.num_nodes),
    }
}

impl From<BlockInfo> for BlockInfoView2 {
    fn from(block_info: BlockInfo) -> Self {
        BlockInfoView2 {
            block_hash: block_info.block_id,
            total_difficulty: block_info.total_difficulty,
            txn_accumulator_info: info1_to_infoview2(&block_info.txn_accumulator_info),
            block_accumulator_info: info1_to_infoview2(&block_info.block_accumulator_info),
            vm_state_accumulator_info: info1_to_infoview2(&block_info.vm_state_accumulator_info),
        }
    }
}
