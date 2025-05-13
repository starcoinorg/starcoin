use std::sync::Arc;

use indexmap::IndexSet;
use itertools::Itertools;
use smallvec::{smallvec, SmallVec};
use starcoin_crypto::HashValue as Hash;
use starcoin_types::{
    block::BlockHeader,
    blockhash::{BlockHashMap, BlockHasher, BlockLevel, ORIGIN},
};

use crate::{
    consensusdb::schemadb::{HeaderStoreReader, ReachabilityStoreReader, RelationsStoreReader},
    reachability::reachability_service::{MTReachabilityService, ReachabilityService},
};

#[derive(Clone)]
pub struct ParentsManager<T: HeaderStoreReader, U: ReachabilityStoreReader, V: RelationsStoreReader>
{
    max_block_level: BlockLevel,
    genesis_hash: Hash,

    headers_store: Arc<T>,
    reachability_service: MTReachabilityService<U>,
    relations_service: V,
}

impl<T: HeaderStoreReader, U: ReachabilityStoreReader, V: RelationsStoreReader>
    ParentsManager<T, U, V>
{
    pub fn new(
        max_block_level: BlockLevel,
        genesis_hash: Hash,
        headers_store: Arc<T>,
        reachability_service: MTReachabilityService<U>,
        relations_service: V,
    ) -> Self {
        Self {
            max_block_level,
            genesis_hash,
            headers_store,
            reachability_service,
            relations_service,
        }
    }

    /// Calculates the parents for each level based on the direct parents. Expects the current
    /// global pruning point s.t. at least one of the direct parents is in its inclusive future
    pub fn calc_block_parents(
        &self,
        current_pruning_point: Hash,
        direct_parents: &[Hash],
    ) -> Vec<Vec<Hash>> {
        let mut direct_parent_headers = direct_parents
            .iter()
            .copied()
            .map(|parent| {
                self.headers_store
                    .get_header_with_block_level(parent)
                    .unwrap()
            })
            .collect_vec();

        // The first candidates to be added should be from a parent in the future of the pruning
        // point, so later on we'll know that every block that doesn't have reachability data
        // (i.e. pruned) is necessarily in the past of the current candidates and cannot be
        // considered as a valid candidate.
        // This is why we sort the direct parent headers in a way that the first one will be
        // in the future of the pruning point.
        let first_parent_in_future_of_pruning_point = direct_parents
            .iter()
            .copied()
            .position(|parent| {
                self.reachability_service
                    .is_dag_ancestor_of(current_pruning_point, parent)
            })
            .expect(
                "at least one of the parents is expected to be in the future of the pruning point",
            );
        direct_parent_headers.swap(0, first_parent_in_future_of_pruning_point);

        let mut origin_children_headers = None;
        let mut parents = Vec::with_capacity(self.max_block_level as usize);

        for block_level in 0..=self.max_block_level {
            // Direct parents are guaranteed to be in one another's anticones so add them all to
            // all the block levels they occupy.
            let mut level_candidates_to_reference_blocks = direct_parent_headers
                .iter()
                .filter(|h| block_level <= h.block_level)
                .map(|h| (h.header.id(), smallvec![h.header.id()]))
                // We use smallvec with size 1 in order to optimize for the common case
                // where the block itself is the only reference block
                .collect::<BlockHashMap<SmallVec<[Hash; 1]>>>();

            // block level = 0, level_candidates_to_reference_blocks = 0,0,1,2
            // block level = 1, level_candidates_to_reference_blocks = 1,2

            let mut first_parent_marker = 0;
            let grandparents = if level_candidates_to_reference_blocks.is_empty() {
                // This means no direct parents at the level, hence we must give precedence to first parent's parents
                // which should all be added as candidates in the processing loop below (since we verified that first
                // parent was in the pruning point's future)
                let mut grandparents = self
                    .parents_at_level(&direct_parent_headers[0].header, block_level)
                    .iter()
                    .copied()
                    // We use IndexSet in order to preserve iteration order and make sure the
                    // processing loop visits the parents of the first parent first
                    .collect::<IndexSet<Hash, BlockHasher>>();
                // Mark the end index of first parent's parents
                first_parent_marker = grandparents.len();
                // Add the remaining level-grandparents
                grandparents.extend(direct_parent_headers[1..].iter().flat_map(|h| {
                    self.parents_at_level(&h.header, block_level)
                        .iter()
                        .copied()
                }));
                grandparents
            } else {
                direct_parent_headers
                    .iter()
                    // We need to iterate parent's parents only if parent is not at block_level
                    .filter(|h| block_level > h.block_level)
                    .flat_map(|h| {
                        self.parents_at_level(&h.header, block_level)
                            .iter()
                            .copied()
                    })
                    .collect::<IndexSet<Hash, BlockHasher>>()
            };

            // block level = 0
            // grandparent = []
            // level_candidates_to_reference_blocks = 0,0,1,2

            // block level = 1
            // grandparent = [0's level 1 parent, 0's level 1 parent]
            // level_candidates_to_reference_blocks = 0,0,1,2

            let parents_at_level = if level_candidates_to_reference_blocks.is_empty()
                && first_parent_marker == grandparents.len()
            {
                // Optimization: this is a common case for high levels where none of the direct parents is on the level
                // and all direct parents have the same level parents. The condition captures this case because all grandparents
                // will be below the first parent marker and there will be no additional grandparents. Bcs all grandparents come
                // from a single, already validated parent, there's no need to run any additional antichain checks and we can return
                // this set.
                grandparents.into_iter().collect()
            } else {
                //
                // Iterate through grandparents in order to find an antichain
                for (i, parent) in grandparents.into_iter().enumerate() {
                    let has_reachability_data =
                        self.reachability_service.has_reachability_data(parent);

                    // Reference blocks are the blocks that are used in reachability queries to check if
                    // a candidate is in the future of another candidate. In most cases this is just the
                    // block itself, but in the case where a block doesn't have reachability data we need
                    // to use some blocks in its future as reference instead.
                    // If we make sure to add a parent in the future of the pruning point first, we can
                    // know that any pruned candidate that is in the past of some blocks in the pruning
                    // point anticone should be a parent (in the relevant level) of one of
                    // the origin children in the pruning point anticone. So we can check which
                    // origin children have this block as parent and use those block as
                    // reference blocks.
                    let reference_blocks = if has_reachability_data {
                        smallvec![parent]
                    } else {
                        // Here we explicitly declare the type because otherwise Rust would make it mutable.
                        let origin_children_headers: &Vec<_> = origin_children_headers.get_or_insert_with(|| {
                            self.relations_service
                                .get_children(Hash::from_slice(ORIGIN).unwrap_or_else(|e| panic!("in calc block parents, failed to change origin: {:?}", e)))
                                .unwrap_or_else(|e| panic!("in calc block parents, origin has no children for: {:?}", e))
                                .iter()
                                .copied()
                                .map(|parent| self.headers_store.get_header(parent).unwrap())
                                .collect_vec()
                        });
                        let mut reference_blocks =
                            SmallVec::with_capacity(origin_children_headers.len());
                        for child_header in origin_children_headers.iter() {
                            if self
                                .parents_at_level(child_header, block_level)
                                .contains(&parent)
                            {
                                reference_blocks.push(child_header.id());
                            }
                        }
                        reference_blocks
                    };

                    // Make sure we process and insert all first parent's parents. See comments above.
                    // Note that as parents of an already validated block, they all form an antichain,
                    // hence no need for reachability queries yet.
                    if i < first_parent_marker {
                        level_candidates_to_reference_blocks.insert(parent, reference_blocks);
                        continue;
                    }

                    if !has_reachability_data {
                        continue;
                    }

                    let len_before_retain = level_candidates_to_reference_blocks.len();
                    level_candidates_to_reference_blocks.retain(|_, refs| {
                        !self
                            .reachability_service
                            .is_any_dag_ancestor(&mut refs.iter().copied(), parent)
                    });
                    let is_any_candidate_ancestor_of =
                        level_candidates_to_reference_blocks.len() < len_before_retain;

                    // We should add the block as a candidate if it's in the future of another candidate
                    // or in the anticone of all candidates.
                    if is_any_candidate_ancestor_of
                        || !level_candidates_to_reference_blocks.iter().any(
                            |(_, candidate_references)| {
                                self.reachability_service.is_dag_ancestor_of_any(
                                    parent,
                                    &mut candidate_references.iter().copied(),
                                )
                            },
                        )
                    {
                        level_candidates_to_reference_blocks.insert(parent, reference_blocks);
                    }
                }

                // After processing all grandparents, collect the successful level candidates
                level_candidates_to_reference_blocks
                    .keys()
                    .copied()
                    .collect_vec()
            };

            if block_level > 0
                && parents_at_level.as_slice() == std::slice::from_ref(&self.genesis_hash)
            {
                break;
            }

            parents.push(parents_at_level);
        }

        parents
    }

    pub fn parents<'a>(
        &'a self,
        header: &'a BlockHeader,
    ) -> impl ExactSizeIterator<Item = &'a [Hash]> {
        (0..=self.max_block_level).map(|level| self.parents_at_level(header, level))
    }

    pub fn parents_at_level<'a>(&'a self, header: &'a BlockHeader, level: u8) -> &'a [Hash] {
        if header.parents_hash().is_empty() {
            // If is genesis
            &[]
        } else if header.parents_hash().len() > level as usize {
            &header.parents_hash[level as usize][..]
        } else {
            std::slice::from_ref(&self.genesis_hash)
        }
    }
}
