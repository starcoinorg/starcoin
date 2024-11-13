use super::util::Refs;
use crate::consensusdb::schemadb::{GhostdagStoreReader, HeaderStoreReader, RelationsStoreReader};
use crate::reachability::reachability_service::ReachabilityService;
use crate::types::{ghostdata::GhostdagData, ordering::*};
use anyhow::{bail, ensure, Context, Result};
use parking_lot::RwLock;
use starcoin_crypto::HashValue as Hash;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockHeader;
use starcoin_types::blockhash::{BlockHashMap, BlockHashes, BlueWorkType, HashKTypeMap, KType};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone)]
pub struct GhostdagManager<
    T: GhostdagStoreReader,
    S: RelationsStoreReader,
    U: ReachabilityService,
    V: HeaderStoreReader,
> {
    pub(super) k: KType,
    pub(super) ghostdag_store: T,
    pub(super) relations_store: Arc<RwLock<S>>,
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
        relations_store: Arc<RwLock<S>>,
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
            genesis.difficulty(),
            genesis.parent_hash(),
            BlockHashes::new(vec![]),
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

    pub fn check_ancestor_of(&self, ancestor: Hash, descendant: Vec<Hash>) -> anyhow::Result<bool> {
        self.reachability_service
            .is_dag_ancestor_of_any_result(ancestor, &mut descendant.into_iter())
            .map_err(|e| e.into())
    }

    pub fn find_selected_parent(
        &self,
        parents: impl IntoIterator<Item = Hash>,
    ) -> anyhow::Result<Hash> {
        parents
            .into_iter()
            .map(|parent| {
                let blue_work = self
                    .ghostdag_store
                    .get_blue_work(parent)
                    .with_context(|| format!("Failed to get blue work for parent {:?}", parent))?;
                Ok(SortableBlock {
                    hash: parent,
                    blue_work,
                })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .max()
            .map(|sortable_block| sortable_block.hash)
            .ok_or_else(|| anyhow::Error::msg("No parent found"))
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
    pub fn ghostdag(&self, parents: &[Hash]) -> Result<GhostdagData> {
        assert!(
            !parents.is_empty(),
            "genesis must be added via a call to init"
        );
        // Run the GHOSTDAG parent selection algorithm
        let selected_parent = self.find_selected_parent(parents.iter().copied())?;
        // Initialize new GHOSTDAG block data with the selected parent
        let mut new_block_data = GhostdagData::new_with_selected_parent(selected_parent, self.k);
        // Get the mergeset in consensus-agreed topological order (topological here means forward in time from blocks to children)
        let ordered_mergeset =
            self.ordered_mergeset_without_selected_parent(selected_parent, parents)?;

        for blue_candidate in ordered_mergeset.iter().cloned() {
            let coloring = self.check_blue_candidate(&new_block_data, blue_candidate)?;
            if let ColoringOutput::Blue(blue_anticone_size, blues_anticone_sizes) = coloring {
                // No k-cluster violation found, we can now set the candidate block as blue
                new_block_data.add_blue(blue_candidate, blue_anticone_size, &blues_anticone_sizes);
            } else {
                new_block_data.add_red(blue_candidate);
            }
        }

        let blue_score = self
            .ghostdag_store
            .get_blue_score(selected_parent)?
            .checked_add(new_block_data.mergeset_blues.len() as u64)
            .expect("blue score size should less than u64");

        let added_blue_work: BlueWorkType = new_block_data
            .mergeset_blues
            .iter()
            .cloned()
            .map(|hash| {
                self.headers_store
                    .get_difficulty(hash)
                    .unwrap_or_else(|err| {
                        error!("Failed to get difficulty of block: {}, {}", hash, err);
                        0.into()
                    })
            })
            .sum();

        let blue_work = self
            .ghostdag_store
            .get_blue_work(selected_parent)?
            .checked_add(added_blue_work)
            .expect("blue work should less than u256");

        new_block_data.finalize_score_and_work(blue_score, blue_work);

        Ok(new_block_data)
    }

    pub(crate) fn verify_and_ghostdata(
        &self,
        blue_blocks: &[BlockHeader],
        header: &BlockHeader,
    ) -> std::result::Result<GhostdagData, anyhow::Error> {
        let parents = header.parents_hash();
        assert!(
            !parents.is_empty(),
            "genesis must be added via a call to init"
        );
        let selected_parent = self.find_selected_parent(header.parents_hash().into_iter())?;
        // Initialize new GHOSTDAG block data with the selected parent
        let mut new_block_data = GhostdagData::new_with_selected_parent(selected_parent, self.k);
        let ordered_mergeset = self.sort_blocks(
            header
                .parents_hash()
                .into_iter()
                .filter(|header_id| *header_id != new_block_data.selected_parent)
                .chain(
                    blue_blocks
                        .iter()
                        .filter(|header| header.id() != new_block_data.selected_parent)
                        .map(|header| header.id()),
                )
                .collect::<HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        )?;

        for blue_candidate in ordered_mergeset.iter().cloned() {
            let coloring = self.check_blue_candidate(&new_block_data, blue_candidate)?;
            if let ColoringOutput::Blue(blue_anticone_size, blues_anticone_sizes) = coloring {
                // No k-cluster violation found, we can now set the candidate block as blue
                new_block_data.add_blue(blue_candidate, blue_anticone_size, &blues_anticone_sizes);
            } else {
                new_block_data.add_red(blue_candidate);
            }
        }

        let remote_blue_set = blue_blocks
            .iter()
            .map(|header| header.id())
            .collect::<HashSet<_>>();
        if new_block_data
            .mergeset_blues
            .iter()
            .skip(1)
            .cloned()
            .collect::<HashSet<_>>()
            != remote_blue_set
        {
            warn!("The data of blue set is not equal when executing the block: {:?}, for {:?}, checking data: {:?}", header.id(), blue_blocks.iter().map(|header| header.id()).collect::<Vec<_>>(), new_block_data.mergeset_blues);
            let ghostdata = self.ghostdag(&header.parents_hash())?;
            if ghostdata
                .mergeset_blues
                .iter()
                .skip(1)
                .cloned()
                .collect::<HashSet<_>>()
                != remote_blue_set
            {
                bail!("The ghost data of blue set is not equal when executing the block: {:?}, for {:?}, checking data: {:?}", header.id(), blue_blocks.iter().map(|header| header.id()).collect::<Vec<_>>(), ghostdata.mergeset_blues);
            }
        }

        let blue_score = self
            .ghostdag_store
            .get_blue_score(selected_parent)?
            .checked_add(new_block_data.mergeset_blues.len() as u64)
            .expect("blue score size should less than u64");

        let added_blue_work: BlueWorkType = new_block_data
            .mergeset_blues
            .iter()
            .cloned()
            .map(|hash| {
                self.headers_store
                    .get_difficulty(hash)
                    .unwrap_or_else(|err| {
                        error!("Failed to get difficulty of block: {}, {}", hash, err);
                        0.into()
                    })
            })
            .sum();

        let blue_work = self
            .ghostdag_store
            .get_blue_work(selected_parent)?
            .checked_add(added_blue_work)
            .expect("blue work should less than u256");

        new_block_data.finalize_score_and_work(blue_score, blue_work);

        info!(
            "verified the block: {:?}, its ghost data: {:?}",
            header.id(),
            new_block_data
        );
        Ok(new_block_data)
    }

    pub fn check_ghostdata_blue_block(&self, ghostdata: &GhostdagData) -> Result<()> {
        let mut check_ghostdata =
            GhostdagData::new_with_selected_parent(ghostdata.selected_parent, self.k);
        for blue_candidate in ghostdata.mergeset_blues.iter().skip(1).cloned() {
            let coloring = self.check_blue_candidate(&check_ghostdata, blue_candidate)?;
            if let ColoringOutput::Blue(blue_anticone_size, blues_anticone_sizes) = coloring {
                check_ghostdata.add_blue(blue_candidate, blue_anticone_size, &blues_anticone_sizes);
            } else {
                check_ghostdata.add_red(blue_candidate);
            }
        }
        if ghostdata.mergeset_blues.len() != check_ghostdata.mergeset_blues.len() {
            return Err(anyhow::anyhow!(
                "The len of blue set is not equal, for {}, checking data: {}",
                ghostdata.mergeset_blues.len(),
                check_ghostdata.mergeset_blues.len()
            ));
        }
        if ghostdata
            .mergeset_blues
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
            != check_ghostdata
                .mergeset_blues
                .iter()
                .cloned()
                .collect::<HashSet<_>>()
        {
            return Err(anyhow::anyhow!("The blue set is not equal"));
        }

        let blue_score = self
            .ghostdag_store
            .get_blue_score(ghostdata.selected_parent)?
            .checked_add(check_ghostdata.mergeset_blues.len() as u64)
            .expect("blue score size should less than u64");

        let added_blue_work: BlueWorkType = check_ghostdata
            .mergeset_blues
            .iter()
            .cloned()
            .map(|hash| {
                self.headers_store
                    .get_difficulty(hash)
                    .unwrap_or_else(|err| {
                        error!("Failed to get difficulty of block: {}, {}", hash, err);
                        0.into()
                    })
            })
            .sum();

        let blue_work = self
            .ghostdag_store
            .get_blue_work(ghostdata.selected_parent)?
            .checked_add(added_blue_work)
            .expect("blue work should less than u256");

        check_ghostdata.finalize_score_and_work(blue_score, blue_work);

        ensure!(
            check_ghostdata.to_compact() == ghostdata.to_compact(),
            "check_ghostdata: {:?} is not the same as ghostdata: {:?}",
            check_ghostdata.to_compact(),
            ghostdata.to_compact()
        );

        Ok(())
    }

    fn check_blue_candidate_with_chain_block(
        &self,
        new_block_data: &GhostdagData,
        chain_block: &ChainBlock,
        blue_candidate: Hash,
        candidate_blues_anticone_sizes: &mut BlockHashMap<KType>,
        candidate_blue_anticone_size: &mut KType,
    ) -> Result<ColoringState> {
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
                return Ok(ColoringState::Blue);
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
                .insert(block, self.blue_anticone_size(block, new_block_data)?);

            *candidate_blue_anticone_size = (*candidate_blue_anticone_size).checked_add(1).unwrap();
            if *candidate_blue_anticone_size > self.k {
                // k-cluster violation: The candidate's blue anticone exceeded k
                info!(
                    "Checking blue candidate: {} failed, blue anticone exceeded k",
                    blue_candidate
                );
                return Ok(ColoringState::Red);
            }

            if *candidate_blues_anticone_sizes.get(&block).unwrap() == self.k {
                // k-cluster violation: A block in candidate's blue anticone already
                // has k blue blocks in its own anticone
                info!(
                    "Checking blue candidate: {} failed, block {} has k blue blocks in its anticone",
                    blue_candidate, block
                );
                return Ok(ColoringState::Red);
            }

            // This is a sanity check that validates that a blue
            // block's blue anticone is not already larger than K.
            assert!(
                *candidate_blues_anticone_sizes.get(&block).unwrap() <= self.k,
                "found blue anticone larger than K"
            );
        }

        Ok(ColoringState::Pending)
    }

    /// Returns the blue anticone size of `block` from the worldview of `context`.
    /// Expects `block` to be in the blue set of `context`
    fn blue_anticone_size(&self, block: Hash, context: &GhostdagData) -> Result<KType> {
        let mut current_blues_anticone_sizes = HashKTypeMap::clone(&context.blues_anticone_sizes);
        let mut current_selected_parent = context.selected_parent;
        loop {
            if let Some(size) = current_blues_anticone_sizes.get(&block) {
                return Ok(*size);
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
                .get_blues_anticone_sizes(current_selected_parent)?;

            current_selected_parent = self
                .ghostdag_store
                .get_selected_parent(current_selected_parent)?;
        }
    }

    pub fn check_blue_candidate(
        &self,
        new_block_data: &GhostdagData,
        blue_candidate: Hash,
    ) -> Result<ColoringOutput> {
        // The maximum length of new_block_data.mergeset_blues can be K+1 because
        // it contains the selected parent.
        if new_block_data.mergeset_blues.len() as KType == self.k.checked_add(1).unwrap() {
            info!(
                "Checking blue candidate: {} failed, mergeset blues size is K+1",
                blue_candidate
            );
            return Ok(ColoringOutput::Red);
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
            )?;

            match state {
                ColoringState::Blue => {
                    return Ok(ColoringOutput::Blue(
                        candidate_blue_anticone_size,
                        candidate_blues_anticone_sizes,
                    ));
                }
                ColoringState::Red => return Ok(ColoringOutput::Red),
                ColoringState::Pending => (), // continue looping
            }

            chain_block = ChainBlock {
                hash: Some(chain_block.data.selected_parent),
                data: self
                    .ghostdag_store
                    .get_data(chain_block.data.selected_parent)
                    .map_err(anyhow::Error::from)?
                    .into(),
            }
        }
    }

    pub fn sort_blocks(&self, blocks: impl IntoIterator<Item = Hash>) -> Result<Vec<Hash>> {
        let mut sorted_blocks: Vec<Hash> = blocks.into_iter().collect();

        sorted_blocks.sort_by_cached_key(|block| {
            let blue_work = self
                .ghostdag_store
                .get_blue_work(*block)
                .unwrap_or_else(|err| {
                    error!("Failed to get blue work of block: {}, {}", *block, err);
                    0.into()
                });
            SortableBlock {
                hash: *block,
                blue_work,
            }
        });
        Ok(sorted_blocks)
    }

    pub fn sort_blocks_for_work_type(
        &self,
        blocks: impl IntoIterator<Item = Hash>,
    ) -> Result<Vec<Hash>> {
        let mut sorted_blocks: Vec<Hash> = blocks.into_iter().collect();
        sorted_blocks.sort_by_cached_key(|block| {
            let blue_work = self
                .ghostdag_store
                .get_blue_work(*block)
                .unwrap_or_else(|err| {
                    error!("Failed to get blue work of block: {}, {}", *block, err);
                    0.into()
                });
            SortableBlockWithWorkType {
                hash: *block,
                blue_work,
            }
        });
        Ok(sorted_blocks)
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
