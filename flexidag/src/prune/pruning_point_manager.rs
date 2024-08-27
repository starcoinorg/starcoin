use starcoin_crypto::HashValue;
use starcoin_logger::prelude::debug;
use starcoin_types::blockhash::ORIGIN;

use crate::reachability::reachability_service::ReachabilityService;
use crate::{
    consensusdb::{
        consenses_state::DagState,
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

    pub fn finality_score(&self, blue_score: u64, pruning_finality: u64) -> u64 {
        blue_score / pruning_finality
    }

    pub fn prune(
        &self,
        dag_state: &DagState,
        current_pruning_point: HashValue,
        next_pruning_point: HashValue,
    ) -> anyhow::Result<Vec<HashValue>> {
        if current_pruning_point == HashValue::zero() {
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
        pruning_point: HashValue,
        ghostdata: &GhostdagData,
        pruning_depth: u64,
        pruning_finality: u64,
    ) -> anyhow::Result<HashValue> {
        let pruning_ghostdata = self.ghost_dag_store.get_data(pruning_point)?;
        let min_required_blue_score_for_next_pruning_point =
            (self.finality_score(pruning_ghostdata.blue_score, pruning_finality) + 1)
                * pruning_finality;

        debug!(
            "min_required_blue_score_for_next_pruning_point: {:?}",
            min_required_blue_score_for_next_pruning_point
        );
        let mut latest_pruning_ghost_data = self.ghost_dag_store.get_compact_data(pruning_point)?;
        if min_required_blue_score_for_next_pruning_point + pruning_depth <= ghostdata.blue_score {
            for child in self.reachability_service().forward_chain_iterator(
                pruning_point,
                ghostdata.selected_parent,
                true,
            ) {
                let next_pruning_ghostdata = self.ghost_dag_store.get_data(child)?;
                debug!(
                    "child: {:?}, observer2.blue_score: {:?}, next_pruning_ghostdata.blue_score: {:?}",
                    child, ghostdata.blue_score, next_pruning_ghostdata.blue_score
                );
                if ghostdata.blue_score - next_pruning_ghostdata.blue_score < pruning_depth {
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

            println!("prune point: {:?}", latest_pruning_ghost_data);
        }

        if latest_pruning_ghost_data.selected_parent == HashValue::new(ORIGIN) {
            anyhow::Ok(pruning_point) // still genesis
        } else {
            anyhow::Ok(latest_pruning_ghost_data.selected_parent)
        }
    }
}
