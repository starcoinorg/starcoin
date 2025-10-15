use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_dag::{
    blockdag::{BlockDAG, PruningPointManager},
    consensusdb::{consensus_state::DagState, prelude::StoreError},
};

pub struct TipManager {
    dag: BlockDAG,
    pruning: PruningPointManager,
    current_pruning: HashValue,
    genesis: HashValue,
}

impl TipManager {
    pub fn new(dag: BlockDAG, genesis: HashValue) -> Result<Self> {
        Ok(Self {
            pruning: dag.pruning_point_manager(),
            dag,
            current_pruning: genesis,
            genesis,
        })
    }

    pub fn current_pruning(&self) -> HashValue {
        self.current_pruning
    }

    pub fn set_pruning_point(&mut self, pruning_point: HashValue) {
        self.current_pruning = if pruning_point == HashValue::zero() {
            self.genesis
        } else {
            pruning_point
        };
    }

    fn load_tips(&self) -> Result<Vec<HashValue>> {
        match self.dag.get_dag_state(self.current_pruning) {
            Ok(state) => Ok(state.tips),
            Err(err) => match err.downcast::<StoreError>()? {
                StoreError::KeyNotFound(_) => Ok(Vec::new()),
                other => Err(other.into()),
            },
        }
    }

    pub fn on_commit(&mut self, block_id: HashValue, parents: &[HashValue]) -> Result<()> {
        let mut tips = self.load_tips()?;
        if tips.is_empty() && !parents.is_empty() {
            tips.extend_from_slice(parents);
        }
        tips.push(block_id);
        tips = tips
            .into_iter()
            .filter(|tip| {
                *tip == block_id || !self.dag.check_ancestor_of(*tip, block_id).unwrap_or(false)
            })
            .collect();
        tips.sort_unstable();
        tips.dedup();
        self.dag
            .save_dag_state(self.current_pruning, DagState { tips })?;
        Ok(())
    }

    pub fn prune_if_needed(
        &mut self,
        previous_pruning: HashValue,
        mut new_pruning: HashValue,
    ) -> Result<Vec<HashValue>> {
        if previous_pruning == new_pruning {
            return Ok(Vec::new());
        }

        let tips = self.dag.get_dag_state(previous_pruning)?.tips;
        let pruned = self.pruning.prune(
            &DagState { tips: tips.clone() },
            previous_pruning,
            new_pruning,
        )?;

        if new_pruning == HashValue::zero() {
            new_pruning = self.genesis;
        }

        self.dag.save_dag_state(
            new_pruning,
            DagState {
                tips: pruned.clone(),
            },
        )?;
        self.current_pruning = new_pruning;
        Ok(pruned)
    }

    pub fn dag(&self) -> &BlockDAG {
        &self.dag
    }
}
