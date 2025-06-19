use crate::{interlink::calculate_level, K_TAIL_BLOCKS, M_PER_LEVEL};
use anyhow::{anyhow, Context, Result};
use starcoin_crypto::HashValue;
use starcoin_types::{
    block::{Block, BlockHeader},
    U256,
};
use std::collections::{HashSet, VecDeque};

/// Metadata for a CompressedProof
#[derive(Clone, Debug)]
pub struct CompressedProofMetadata {
    /// The hash of the most recent pruning point on the selected chain
    pub pruning_block_hash: HashValue,

    /// The block height of that pruning point
    pub pruning_block_height: u64,

    /// Number of tail blocks included in the proof
    pub tail_block_count: u16,

    /// Lower‐bound on total difficulty as recomputed from superblocks
    pub total_difficulty: U256,
}

/// The full compressed‐proof sent to node or stored on disk
#[derive(Clone, Debug)]
pub struct CompressedProof {
    /// Proof metadata (pruning point info, window sizes, total difficulty)
    pub meta: CompressedProofMetadata,

    /// Sliding window of epoch difficulties ending at the pruning point
    pub superblock_headers: Vec<BlockHeader>,

    /// All blue block headers referenced by those superblocks
    pub blueblock_headers: Vec<BlockHeader>,

    /// The last k full blocks (𝜒 tail) with bodies, starting from the pruning point
    pub tail_blocks: Vec<Block>,
}

/// State for building and updating a compressed proof of chain work and structure
pub struct CompressedProofState {
    /// For each superblock level, keep a rolling queue of the most recent 2·m headers
    pub level_buckets: Vec<VecDeque<BlockHeader>>,
    /// Flat, height-ordered list of all stable superblock headers
    pub superblock_headers: Vec<BlockHeader>,
    /// Set of blue block hashes seen in those superblocks (to avoid duplicates)
    pub blueblock_hashes: HashSet<HashValue>,
    /// Unstable tail of recent block hashes (length ≤ K_TAIL_BLOCKS)
    pub tail_block_hashes: VecDeque<HashValue>,
    /// A provable lower bound of total chain difficulty, recomputed from superblocks
    pub total_difficulty: u128,
}

impl CompressedProofState {
    /// Initialize from a previously-verified compressed proof snapshot
    pub fn initialize_from_proof(proof: &CompressedProof) -> Result<Self> {
        // Determine highest superblock level based on pow_hash and difficulty
        let levels: Vec<u8> = proof
            .superblock_headers
            .iter()
            .map(|h| calculate_level(h.id(), h.difficulty()))
            .collect::<Result<_>>()?;
        let max_level = levels
            .into_iter()
            .max()
            .ok_or_else(|| anyhow!("compressed proof contains no superblock headers"))?
            as usize;

        // Build empty per-level buckets
        let mut level_buckets = Vec::with_capacity(max_level + 1);
        for _ in 0..=max_level {
            level_buckets.push(VecDeque::new());
        }

        // Populate each bucket with the last 2·m superblocks of that level
        for header in &proof.superblock_headers {
            let lvl_u8 = calculate_level(header.id(), header.difficulty())
                .context("failed to calculate level during initialization")?;
            let lvl = lvl_u8 as usize;
            let bucket = &mut level_buckets
                .get_mut(lvl)
                .with_context(|| format!("level {} out of bounds for buckets", lvl))?;
            bucket.push_back(header.clone());
            if bucket.len() > 2 * M_PER_LEVEL as usize {
                bucket.pop_front();
            }
        }

        // Clone the flat list and extract all referenced blue hashes
        let superblock_headers = proof.superblock_headers.clone();
        let mut blueblock_hashes = HashSet::new();
        for header in &superblock_headers {
            for &blue in &header.blue_hashes {
                blueblock_hashes.insert(blue);
            }
        }

        // Record the unstable suffix of block hashes
        let tail_block_hashes = proof
            .tail_blocks
            .iter()
            .map(|blk| blk.header.hash())
            .collect::<VecDeque<_>>();

        // Compute a provable lower-bound on total difficulty from the superblocks
        let total_difficulty = recompute_total_difficulty(&superblock_headers)?;

        Ok(CompressedProofState {
            level_buckets,
            superblock_headers,
            blueblock_hashes,
            tail_block_hashes,
            total_difficulty,
        })
    }

    /// Incorporate a new block header into the proof state incrementally
    pub fn on_new_block(&mut self, header: BlockHeader, store: &dyn ChainStore) -> Result<()> {
        // Append new header's hash to the tail
        self.tail_block_hashes.push_back(header.hash());
        // If the tail exceeds K_TAIL_BLOCKS, evict the oldest block
        if self.tail_block_hashes.len() > K_TAIL_BLOCKS as usize {
            let old_hash = self.tail_block_hashes.pop_front().unwrap();
            let old_block = store
                .get_block(old_hash)
                .context("failed to fetch evicted block from store")?;
            self.promote_to_superblock(&old_block.header)?;
        }
        // Recompute total difficulty after any promotion
        self.total_difficulty = recompute_total_difficulty(&self.superblock_headers)?;
        Ok(())
    }

    /// When a block leaves the unstable tail, promote it to the stable superblock set
    fn promote_to_superblock(&mut self, header: &BlockHeader) -> Result<()> {
        // Determine level by recalculating from pow_hash and difficulty
        let lvl_u8 = calculate_level(header.pow_hash, header.difficulty())
            .context("failed to calculate level during promotion")?;
        let lvl = lvl_u8 as usize;
        let bucket = self
            .level_buckets
            .get_mut(lvl)
            .with_context(|| format!("level {} out of bounds for promotion", lvl))?;
        bucket.push_back(header.clone());
        if bucket.len() > 2 * M_PER_LEVEL as usize {
            bucket.pop_front();
        }
        // Append to flat superblock list
        self.superblock_headers.push(header.clone());
        // Track its blue references
        for &blue_hash in &header.blue_hashes {
            self.blueblock_hashes.insert(blue_hash);
        }
        Ok(())
    }

    /// Export a new CompressedProof snapshot for distribution or storage
    pub fn export_compressed_proof(&self, store: &dyn ChainStore) -> Result<CompressedProof> {
        // The last tail block's header carries the pruning point
        let last_hash = *self
            .tail_block_hashes
            .back()
            .expect("tail must not be empty");
        let last_block = store
            .get_block(last_hash)
            .context("failed to fetch last block from store")?;
        let meta = CompressedProofMetadata {
            pruning_block_hash: last_block.header.pruning_point_hash,
            pruning_block_height: last_block.header.pruning_point_height,
            tail_block_count: self.tail_block_hashes.len() as u16,
            total_difficulty: self.total_difficulty,
        };

        // Gather full tail blocks
        let tail_blocks = self
            .tail_block_hashes
            .iter()
            .map(|&h| store.get_block(h).context("failed to fetch tail block"))
            .collect::<Result<Vec<_>>>()?;

        // Gather headers of all blue blocks referenced by superblocks
        let blueblock_headers = self
            .superblock_headers
            .iter()
            .flat_map(|h| h.blue_hashes.iter())
            .map(|&hash| {
                store
                    .get_header(hash)
                    .context("failed to fetch blue header")
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(CompressedProof {
            meta,
            superblock_headers: self.superblock_headers.clone(),
            blueblock_headers,
            tail_blocks,
        })
    }
}

/// Compute a provable lower-bound on total difficulty solely from superblock levels
///  
/// We recalculate each header's level from its pow_hash and difficulty, count how many
/// superblocks have level ≥ µ, then take differences to get exact counts
/// and sum count(µ)·2^µ as a succinct PoW lower bound.
///
fn recompute_total_difficulty(superblocks: &[BlockHeader]) -> Result<u128> {
    // Determine maximum observed level from pow_hash and difficulty
    let max_level = superblocks
        .iter()
        .map(|h| {
            calculate_level(h.id(), h.difficulty())
                .context("failed to calculate level during recompute")
        })
        .collect::<Result<Vec<u8>>>()?
        .into_iter()
        .max()
        .ok_or_else(|| anyhow!("no superblocks to recompute difficulty"))?
        as usize;

    // counts[i] = number of superblocks with recalculated level ≥ i
    let mut counts = vec![0usize; max_level + 2];
    for header in superblocks {
        let lvl = calculate_level(header.id(), header.difficulty())
            .context("failed to calculate level during recompute")? as usize;
        for i in 0..=lvl {
            counts[i] += 1;
        }
    }
    // Sum differences * 2^level
    let mut total = 0u128;
    for i in 0..=max_level {
        let di = counts[i].saturating_sub(counts[i + 1]);
        total += (1u128 << i) * (di as u128);
    }
    Ok(total)
}
