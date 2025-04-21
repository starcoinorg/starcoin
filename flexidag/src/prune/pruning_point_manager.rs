use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;

use crate::reachability::reachability_service::ReachabilityService;
use crate::{
    consensusdb::{
        consensus_state::DagState,
        schemadb::{DbGhostdagStore, GhostdagStoreReader, ReachabilityStoreReader},
    },
    reachability::reachability_service::MTReachabilityService,
    types::ghostdata::{CompactGhostdagData, GhostdagData},
};

#[derive(Clone)]
pub struct PruningPointManagerT<T: ReachabilityStoreReader + Clone> {
    reachability_service: MTReachabilityService<T>,
    ghost_dag_store: DbGhostdagStore,
}

impl<T: ReachabilityStoreReader + Clone> PruningPointManagerT<T> {
    pub fn new(
        reachability_service: MTReachabilityService<T>,
        ghost_dag_store: DbGhostdagStore,
    ) -> Self {
        Self {
            reachability_service,
            ghost_dag_store,
        }
    }

    pub fn reachability_service(&self) -> MTReachabilityService<T> {
        self.reachability_service.clone()
    }

    pub(crate) fn finality_score(&self, blue_score: u64, pruning_finality: u64) -> u64 {
        blue_score / pruning_finality
    }

    pub fn prune(
        &self,
        dag_state: &DagState,
        current_pruning_point: HashValue,
        next_pruning_point: HashValue,
    ) -> anyhow::Result<Vec<HashValue>> {
        if next_pruning_point == HashValue::zero() {
            return Ok(dag_state.tips.clone());
        }
        if current_pruning_point == next_pruning_point {
            return Ok(dag_state.tips.clone());
        }
        anyhow::Ok(
            dag_state
                .tips
                .iter()
                .filter(|tip| {
                    self.reachability_service
                        .is_dag_ancestor_of(next_pruning_point, **tip)
                })
                .cloned()
                .collect(),
        )
    }

    pub(crate) fn next_pruning_point(
        &self,
        previous_pruning_point: HashValue,
        previous_ghostdata: &GhostdagData,
        next_ghostdata: &GhostdagData,
        pruning_depth: u64,
        pruning_finality: u64,
    ) -> anyhow::Result<HashValue> {
        info!(
            "previous_pruning_point: {:?}, previous_ghostdata: {:?}, next_ghostdata: {:?}, pruning_depth: {}, pruning_finality: {}",
            previous_pruning_point, previous_ghostdata.to_compact(), next_ghostdata.to_compact(), pruning_depth, pruning_finality
        );
        let min_required_blue_score_for_next_pruning_point =
            (self.finality_score(previous_ghostdata.blue_score, pruning_finality) + 1)
                * pruning_finality;

        let mut latest_pruning_ghost_data = previous_ghostdata.to_compact();
        if min_required_blue_score_for_next_pruning_point + pruning_depth
            <= next_ghostdata.blue_score
        {
            for child in self.reachability_service().forward_chain_iterator(
                previous_pruning_point,
                next_ghostdata.selected_parent,
                true,
            ) {
                let next_pruning_ghostdata = self.ghost_dag_store.get_data(child)?;
                if next_ghostdata.blue_score - next_pruning_ghostdata.blue_score < pruning_depth {
                    break;
                }

                if self.finality_score(next_pruning_ghostdata.blue_score, pruning_finality)
                    > self.finality_score(latest_pruning_ghost_data.blue_score, pruning_finality)
                {
                    latest_pruning_ghost_data = CompactGhostdagData {
                        blue_score: next_pruning_ghostdata.blue_score,
                        blue_work: next_pruning_ghostdata.blue_work,
                        selected_parent: next_pruning_ghostdata.selected_parent,
                    };
                }
            }

            info!("prune point: {:?}", latest_pruning_ghost_data);
        }

        if latest_pruning_ghost_data.selected_parent
            == previous_ghostdata.to_compact().selected_parent
        {
            anyhow::Ok(previous_pruning_point)
        } else {
            anyhow::Ok(latest_pruning_ghost_data.selected_parent)
        }
    }
}
