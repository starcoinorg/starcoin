use std::sync::Arc;

use starcoin_crypto::HashValue;

use crate::{
    consensusdb::{
        consensus_block_depth::BlockDepthInfoReader,
        schemadb::{GhostdagStoreReader, ReachabilityStoreReader},
    },
    reachability::reachability_service::{MTReachabilityService, ReachabilityService},
    types::ghostdata::GhostdagData,
};

#[derive(Clone)]
pub struct BlockDepthManagerT<
    S: BlockDepthInfoReader,
    U: ReachabilityStoreReader,
    V: GhostdagStoreReader,
> {
    depth_store: Arc<S>,
    reachability_service: MTReachabilityService<U>,
    ghostdag_store: V,
}

impl<S: BlockDepthInfoReader, U: ReachabilityStoreReader, V: GhostdagStoreReader>
    BlockDepthManagerT<S, U, V>
{
    pub fn new(
        depth_store: Arc<S>,
        reachability_service: MTReachabilityService<U>,
        ghostdag_store: V,
    ) -> Self {
        Self {
            depth_store,
            reachability_service,
            ghostdag_store,
        }
    }

    pub fn calc_merge_depth_root(
        &self,
        ghostdag_data: &GhostdagData,
        pruning_point: HashValue,
        merge_depth: u64,
    ) -> anyhow::Result<HashValue> {
        self.calculate_block_at_depth(ghostdag_data, merge_depth, pruning_point)
    }

    pub fn calc_finality_point(
        &self,
        ghostdag_data: &GhostdagData,
        pruning_point: HashValue,
        finality_depth: u64,
    ) -> anyhow::Result<HashValue> {
        self.calculate_block_at_depth(ghostdag_data, finality_depth, pruning_point)
    }

    // return hash zero if no requiring merge depth
    fn calculate_block_at_depth(
        &self,
        ghostdag_data: &GhostdagData,
        depth: u64,
        pruning_point: HashValue,
    ) -> anyhow::Result<HashValue> {
        if ghostdag_data.blue_score < depth {
            return anyhow::Ok(HashValue::zero());
        }

        let pp_bs = self.ghostdag_store.get_blue_score(pruning_point)?;

        if ghostdag_data.blue_score < pp_bs + depth {
            return anyhow::Ok(HashValue::zero());
        }

        if !self
            .reachability_service
            .is_chain_ancestor_of(pruning_point, ghostdag_data.selected_parent)
        {
            return anyhow::Ok(HashValue::zero());
        }

        let mut current = match self
            .depth_store
            .get_block_depth_info(ghostdag_data.selected_parent)?
        {
            Some(block_depth_info) => block_depth_info.merge_depth_root,
            None => HashValue::zero(),
        };

        if current == HashValue::zero() {
            current = pruning_point;
        }

        let required_blue_score = ghostdag_data.blue_score - depth;

        for chain_block in self.reachability_service.forward_chain_iterator(
            current,
            ghostdag_data.selected_parent,
            true,
        ) {
            if self.ghostdag_store.get_blue_score(chain_block)? >= required_blue_score {
                break;
            }

            current = chain_block;
        }

        anyhow::Ok(current)
    }

    /// Returns the set of blues which are eligible for "kosherizing" merge bound violating blocks.
    /// By prunality rules, these blocks must have `merge_depth_root` on their selected chain.  
    pub fn kosherizing_blues<'a>(
        &'a self,
        ghostdag_data: &'a GhostdagData,
        merge_depth_root: HashValue,
    ) -> impl DoubleEndedIterator<Item = HashValue> + 'a {
        ghostdag_data
            .mergeset_blues
            .iter()
            .copied()
            .filter(move |blue| {
                self.reachability_service
                    .is_chain_ancestor_of(merge_depth_root, *blue)
            })
    }
}
