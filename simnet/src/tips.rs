use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_dag::{
    blockdag::{BlockDAG, PruningPointManager},
    consensusdb::prelude::StoreError,
};

/// Minimal tip manager mirroring chain::renew_tips logic for simnet.
pub struct TipManager {
    dag: BlockDAG,
    pruning_manager: PruningPointManager,
    genesis: HashValue,
    current_pruning: HashValue,
}

impl TipManager {
    pub fn new(dag: BlockDAG, genesis: HashValue) -> Result<Self> {
        let pruning_manager = dag.pruning_point_manager();
        Ok(Self {
            dag,
            pruning_manager,
            genesis,
            current_pruning: genesis,
        })
    }

    pub fn set_pruning_point(&mut self, pruning: HashValue) {
        self.current_pruning = if pruning == HashValue::zero() {
            self.genesis
        } else {
            pruning
        };
    }

    fn load_tips(&self) -> Result<Vec<HashValue>> {
        match self.dag.get_dag_state(self.current_pruning) {
            Ok(state) => Ok(state.tips),
            Err(e) => match e.downcast::<StoreError>()? {
                StoreError::KeyNotFound(_) => Ok(Vec::new()),
                other => Err(other.into()),
            },
        }
    }

    pub fn update_tips(&mut self, block_id: HashValue, parents: &[HashValue]) -> Result<()> {
        let mut tips = self.load_tips()?;
        if tips.is_empty() {
            tips = parents.to_vec();
        }

        let mut filtered = Vec::new();
        for tip in tips {
            if tip != block_id && !self.dag.check_ancestor_of(tip, block_id)? {
                filtered.push(tip);
            }
        }
        filtered.push(block_id);

        self.dag.save_dag_state(
            self.current_pruning,
            starcoin_dag::consensusdb::consensus_state::DagState { tips: filtered },
        )?;
        Ok(())
    }

    pub fn prune_if_needed(
        &mut self,
        previous_pruning: HashValue,
        new_pruning: HashValue,
    ) -> Result<()> {
        if previous_pruning == new_pruning {
            return Ok(());
        }

        let tips_before = self.dag.get_dag_state(self.current_pruning)?.tips;
        let pruned = self.pruning_manager.prune(
            &starcoin_dag::consensusdb::consensus_state::DagState { tips: tips_before },
            previous_pruning,
            new_pruning,
        )?;

        let pruning_point = if new_pruning == HashValue::zero() {
            self.genesis
        } else {
            new_pruning
        };

        self.dag.save_dag_state(
            pruning_point,
            starcoin_dag::consensusdb::consensus_state::DagState { tips: pruned },
        )?;
        self.current_pruning = pruning_point;
        Ok(())
    }
}
