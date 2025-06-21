use crate::interlink::calculate_level;
use anyhow::Result;
use starcoin_types:: block::BlockHeader;
use std::collections::VecDeque;
use starcoin_chain_api::ReadableChainService;

/// Algorithm 1's Dissolveₘ,ₖ(C)
/// Collects the last 2*m superblocks per level from the new pruning point.
pub fn dissolve_m_k(
    chain: &dyn ReadableChainService,
    pruning_point: &BlockHeader,
    m: usize,
) -> Result<Vec<VecDeque<BlockHeader>>> {

    // Compute the block's own level
    let new_level = calculate_level(pruning_point.id(), pruning_point.difficulty())? as usize;
    
    // Determine interlink capacity
    let old_max_level = pruning_point.interlink().len().saturating_sub(1);
    
    // Use the maximum to ensure the new pruning point's level is considered
    let max_level = std::cmp::max(new_level, old_max_level);

    // Initialize a VecDeque for each level up to max_level
    let mut d_u: Vec<VecDeque<BlockHeader>> = Vec::with_capacity(max_level + 1);
    for _ in 0..=max_level {
        d_u.push(VecDeque::new());
    }

    // For each level u, traverse interlink[u] backwards and collect up to 2*m blocks
    for level in 0..=max_level {
	
        let mut maybe_header = Some(pruning_point.clone());
        let bucket = &mut d_u[level];

        while bucket.len() < 2 * m {
            let block_header = match maybe_header.take() {
                Some(h) => h,
                None => break, // Reached genesis or no more superblocks
            };

            // Only collect headers whose level matches level
            let block_level = calculate_level(block_header.id(), block_header.difficulty())? as usize;
            if block_level == level {
                bucket.push_front(block_header.clone());
            }
            // Jump to the previous superblock at level u
            maybe_header = chain.get_header_by_hash(block_header.interlink()[level])?; // propagate errors
	}
    }
    Ok(d_u)
}
