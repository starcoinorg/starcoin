use super::util::Refs;
use crate::consensusdb::schemadb::{GhostdagStoreReader, HeaderStoreReader, RelationsStoreReader};
use crate::dag::reachability::reachability_service::ReachabilityService;
use crate::dag::types::{ghostdata::GhostdagData, ordering::*};
use starcoin_crypto::HashValue as Hash;
use starcoin_types::block::BlockHeader;
use starcoin_types::blockhash::{BlockHashMap, BlockHashes, BlueWorkType, HashKTypeMap, KType};
use starcoin_types::U256;
use std::sync::Arc;
// For GhostdagStoreReader-related functions, use GhostDagDataWrapper instead.
//  ascending_mergeset_without_selected_parent
//  descending_mergeset_without_selected_parent
//  consensus_ordered_mergeset
//  consensus_ordered_mergeset_without_selected_parent
//use dag_database::consensus::GhostDagDataWrapper;

#[derive(Clone)]
pub struct GhostdagManager<
    T: GhostdagStoreReader,
    S: RelationsStoreReader,
    U: ReachabilityService,
    V: HeaderStoreReader,
> {
    pub(super) k: KType,
    pub(super) ghostdag_store: T,
    pub(super) relations_store: S,
    pub(super) headers_store: V,
    pub(super) reachability_service: U,
}

impl<
        T: GhostdagStoreReader,
        S: RelationsStoreReader,
        U: ReachabilityService,
        V: HeaderStoreReader,
    > GhostdagManager<T, S, U, V>
{
    pub fn new(
        k: KType,
        ghostdag_store: T,
        relations_store: S,
        headers_store: V,
        reachability_service: U,
    ) -> Self {
        Self {
            k,
            ghostdag_store,
            relations_store,
            reachability_service,
            headers_store,
        }
    }

    pub fn genesis_ghostdag_data(&self, genesis: &BlockHeader) -> GhostdagData {
        GhostdagData::new(
            0,
            Default::default(), //todo:: difficulty
            genesis.parent_hash(),
            BlockHashes::new(Vec::new()),
            BlockHashes::new(Vec::new()),
            HashKTypeMap::new(BlockHashMap::new()),
        )
    }

    pub fn origin_ghostdag_data(&self) -> Arc<GhostdagData> {
        Arc::new(GhostdagData::new(
            0,
            Default::default(),
            0.into(),
            BlockHashes::new(Vec::new()),
            BlockHashes::new(Vec::new()),
            HashKTypeMap::new(BlockHashMap::new()),
        ))
    }

    pub fn find_selected_parent(&self, parents: impl IntoIterator<Item = Hash>) -> Hash {
        parents
            .into_iter()
            .map(|parent| SortableBlock {
                hash: parent,
                blue_work: self.ghostdag_store.get_blue_work(parent).unwrap(),
            })
            .max()
            .unwrap()
            .hash
    }

    /// Runs the GHOSTDAG protocol and calculates the block GhostdagData by the given parents.
    /// The function calculates mergeset blues by iterating over the blocks in
    /// the anticone of the new block selected parent (which is the parent with the
    /// highest blue work) and adds any block to the blue set if by adding
    /// it these conditions will not be violated:
    ///
    /// 1) |anticone-of-candidate-block ∩ blue-set-of-new-block| ≤ K
    ///
    /// 2) For every blue block in blue-set-of-new-block:
    ///    |(anticone-of-blue-block ∩ blue-set-new-block) ∪ {candidate-block}| ≤ K.
    ///    We validate this condition by maintaining a map blues_anticone_sizes for
    ///    each block which holds all the blue anticone sizes that were affected by
    ///    the new added blue blocks.
    ///    So to find out what is |anticone-of-blue ∩ blue-set-of-new-block| we just iterate in
    ///    the selected parent chain of the new block until we find an existing entry in
    ///    blues_anticone_sizes.
    ///
    /// For further details see the article https://eprint.iacr.org/2018/104.pdf
    pub fn ghostdag(&self, parents: &[Hash]) -> GhostdagData {
        assert!(
            !parents.is_empty(),
            "genesis must be added via a call to init"
        );

        // Run the GHOSTDAG parent selection algorithm
        let selected_parent = self.find_selected_parent(&mut parents.iter().copied());
        // Initialize new GHOSTDAG block data with the selected parent
        let mut new_block_data = GhostdagData::new_with_selected_parent(selected_parent, self.k);
        // Get the mergeset in consensus-agreed topological order (topological here means forward in time from blocks to children)
        let ordered_mergeset =
            self.ordered_mergeset_without_selected_parent(selected_parent, parents);

        for blue_candidate in ordered_mergeset.iter().cloned() {
            let coloring = self.check_blue_candidate(&new_block_data, blue_candidate);

            if let ColoringOutput::Blue(blue_anticone_size, blues_anticone_sizes) = coloring {
                // No k-cluster violation found, we can now set the candidate block as blue
                new_block_data.add_blue(blue_candidate, blue_anticone_size, &blues_anticone_sizes);
            } else {
                new_block_data.add_red(blue_candidate);
            }
        }

        let blue_score = self
            .ghostdag_store
            .get_blue_score(selected_parent)
            .unwrap()
            .checked_add(new_block_data.mergeset_blues.len() as u64)
            .unwrap();

        let added_blue_work: BlueWorkType = new_block_data
            .mergeset_blues
            .iter()
            .cloned()
            .map(|hash| self.headers_store.get_difficulty(hash).unwrap_or(0.into()))
            .sum();

        let blue_work = self
            .ghostdag_store
            .get_blue_work(selected_parent)
            .unwrap()
            .checked_add(added_blue_work)
            .unwrap();
        new_block_data.finalize_score_and_work(blue_score, blue_work);

        new_block_data
    }

    fn check_blue_candidate_with_chain_block(
        &self,
        new_block_data: &GhostdagData,
        chain_block: &ChainBlock,
        blue_candidate: Hash,
        candidate_blues_anticone_sizes: &mut BlockHashMap<KType>,
        candidate_blue_anticone_size: &mut KType,
    ) -> ColoringState {
        // If blue_candidate is in the future of chain_block, it means
        // that all remaining blues are in the past of chain_block and thus
        // in the past of blue_candidate. In this case we know for sure that
        // the anticone of blue_candidate will not exceed K, and we can mark
        // it as blue.
        //
        // The new block is always in the future of blue_candidate, so there's
        // no point in checking it.

        // We check if chain_block is not the new block by checking if it has a hash.
        if let Some(hash) = chain_block.hash {
            if self
                .reachability_service
                .is_dag_ancestor_of(hash, blue_candidate)
            {
                return ColoringState::Blue;
            }
        }

        for &block in chain_block.data.mergeset_blues.iter() {
            // Skip blocks that exist in the past of blue_candidate.
            if self
                .reachability_service
                .is_dag_ancestor_of(block, blue_candidate)
            {
                continue;
            }

            candidate_blues_anticone_sizes
                .insert(block, self.blue_anticone_size(block, new_block_data));

            *candidate_blue_anticone_size = (*candidate_blue_anticone_size).checked_add(1).unwrap();
            if *candidate_blue_anticone_size > self.k {
                // k-cluster violation: The candidate's blue anticone exceeded k
                return ColoringState::Red;
            }

            if *candidate_blues_anticone_sizes.get(&block).unwrap() == self.k {
                // k-cluster violation: A block in candidate's blue anticone already
                // has k blue blocks in its own anticone
                return ColoringState::Red;
            }

            // This is a sanity check that validates that a blue
            // block's blue anticone is not already larger than K.
            assert!(
                *candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k,
                "found blue anticone larger than K"
            );
        }

        ColoringState::Pending
    }

    /// Returns the blue anticone size of `block` from the worldview of `context`.
    /// Expects `block` to be in the blue set of `context`
    fn blue_anticone_size(&self, block: Hash, context: &GhostdagData) -> KType {
        let mut current_blues_anticone_sizes = HashKTypeMap::clone(&context.blues_anticone_sizes);
        let mut current_selected_parent = context.selected_parent;
        loop {
            if let Some(size) = current_blues_anticone_sizes.get(&block) {
                return *size;
            }
            /* TODO: consider refactor it
            if current_selected_parent == self.genesis_hash
                || current_selected_parent == Hash::new(blockhash::ORIGIN)
            {
                panic!("block {block} is not in blue set of the given context");
            }
            */
            current_blues_anticone_sizes = self
                .ghostdag_store
                .get_blues_anticone_sizes(current_selected_parent)
                .unwrap();
            current_selected_parent = self
                .ghostdag_store
                .get_selected_parent(current_selected_parent)
                .unwrap();
        }
    }

    pub fn check_blue_candidate(
        &self,
        new_block_data: &GhostdagData,
        blue_candidate: Hash,
    ) -> ColoringOutput {
        // The maximum length of new_block_data.mergeset_blues can be K+1 because
        // it contains the selected parent.
        if new_block_data.mergeset_blues.len() as KType == self.k.checked_add(1).unwrap() {
            return ColoringOutput::Red;
        }

        let mut candidate_blues_anticone_sizes: BlockHashMap<KType> =
            BlockHashMap::with_capacity(self.k as usize);
        // Iterate over all blocks in the blue past of the new block that are not in the past
        // of blue_candidate, and check for each one of them if blue_candidate potentially
        // enlarges their blue anticone to be over K, or that they enlarge the blue anticone
        // of blue_candidate to be over K.
        let mut chain_block = ChainBlock {
            hash: None,
            data: new_block_data.into(),
        };
        let mut candidate_blue_anticone_size: KType = 0;

        loop {
            let state = self.check_blue_candidate_with_chain_block(
                new_block_data,
                &chain_block,
                blue_candidate,
                &mut candidate_blues_anticone_sizes,
                &mut candidate_blue_anticone_size,
            );

            match state {
                ColoringState::Blue => {
                    return ColoringOutput::Blue(
                        candidate_blue_anticone_size,
                        candidate_blues_anticone_sizes,
                    );
                }
                ColoringState::Red => return ColoringOutput::Red,
                ColoringState::Pending => (), // continue looping
            }

            chain_block = ChainBlock {
                hash: Some(chain_block.data.selected_parent),
                data: self
                    .ghostdag_store
                    .get_data(chain_block.data.selected_parent)
                    .unwrap()
                    .into(),
            }
        }
    }

    pub fn sort_blocks(&self, blocks: impl IntoIterator<Item = Hash>) -> Vec<Hash> {
        let mut sorted_blocks: Vec<Hash> = blocks.into_iter().collect();
        sorted_blocks.sort_by_cached_key(|block| SortableBlock {
            hash: *block,
            blue_work: self.ghostdag_store.get_blue_work(*block).unwrap(),
        });
        sorted_blocks
    }
}

/// Chain block with attached ghostdag data
struct ChainBlock<'a> {
    hash: Option<Hash>,
    // if set to `None`, signals being the new block
    data: Refs<'a, GhostdagData>,
}

/// Represents the intermediate GHOSTDAG coloring state for the current candidate
enum ColoringState {
    Blue,
    Red,
    Pending,
}

#[derive(Debug)]
/// Represents the final output of GHOSTDAG coloring for the current candidate
pub enum ColoringOutput {
    Blue(KType, BlockHashMap<KType>),
    // (blue anticone size, map of blue anticone sizes for each affected blue)
    Red,
}
