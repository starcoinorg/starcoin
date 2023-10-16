use super::protocol::GhostdagManager;
use crate::consensusdb::schemadb::{GhostdagStoreReader, HeaderStoreReader, RelationsStoreReader};
use crate::dag::reachability::reachability_service::ReachabilityService;
use starcoin_crypto::HashValue as Hash;
use starcoin_types::blockhash::BlockHashSet;
use std::collections::VecDeque;

impl<
        T: GhostdagStoreReader,
        S: RelationsStoreReader,
        U: ReachabilityService,
        V: HeaderStoreReader,
    > GhostdagManager<T, S, U, V>
{
    pub fn ordered_mergeset_without_selected_parent(
        &self,
        selected_parent: Hash,
        parents: &[Hash],
    ) -> Vec<Hash> {
        self.sort_blocks(self.unordered_mergeset_without_selected_parent(selected_parent, parents))
    }

    pub fn unordered_mergeset_without_selected_parent(
        &self,
        selected_parent: Hash,
        parents: &[Hash],
    ) -> BlockHashSet {
        let mut queue: VecDeque<_> = parents
            .iter()
            .copied()
            .filter(|p| p != &selected_parent)
            .collect();
        let mut mergeset: BlockHashSet = queue.iter().copied().collect();
        let mut selected_parent_past = BlockHashSet::new();

        while let Some(current) = queue.pop_front() {
            let current_parents = self
                .relations_store
                .get_parents(current)
                .unwrap_or_else(|err| {
                    println!("WUT");
                    panic!("{err:?}");
                });

            // For each parent of the current block we check whether it is in the past of the selected parent. If not,
            // we add it to the resulting merge-set and queue it for further processing.
            for parent in current_parents.iter() {
                if mergeset.contains(parent) {
                    continue;
                }

                if selected_parent_past.contains(parent) {
                    continue;
                }

                if self
                    .reachability_service
                    .is_dag_ancestor_of(*parent, selected_parent)
                {
                    selected_parent_past.insert(*parent);
                    continue;
                }

                mergeset.insert(*parent);
                queue.push_back(*parent);
            }
        }

        mergeset
    }
}
