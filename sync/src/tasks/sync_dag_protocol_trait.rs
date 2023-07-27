use anyhow::Result;
use futures::future::BoxFuture;
use network_p2p_core::PeerId;
use starcoin_network_rpc_api::dag_protocol::{
    SyncDagBlockInfo, TargetDagAccumulatorLeaf, TargetDagAccumulatorLeafDetail,
};

pub trait PeerSynDagAccumulator: Send + Sync {
    fn get_sync_dag_asccumulator_leaves(
        &self,
        peer_id: Option<PeerId>,
        leaf_index: u64,
        batch_size: u64,
    ) -> BoxFuture<Result<Vec<TargetDagAccumulatorLeaf>>>;

    fn get_accumulator_leaf_detail(
        &self,
        peer_id: Option<PeerId>,
        leaf_index: u64,
        batch_size: u64,
    ) -> BoxFuture<Result<Option<Vec<TargetDagAccumulatorLeafDetail>>>>;

    fn get_dag_block_info(
        &self,
        peer: Option<PeerId>,
        leaf_index: u64,
        batch_size: u64,
    ) -> BoxFuture<Result<Option<Vec<SyncDagBlockInfo>>>>;
}
