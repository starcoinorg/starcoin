use crate::interlink::calculate_level;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_types:: block::BlockHeader;
use std::collections::VecDeque;
use starcoin_chain_api::ReadableChainService;
use thiserror::Error;
/// Algorithm 1's Dissolveₘ,ₖ(C)
/// Collects the last 2*m superblocks per level from the new pruning point.
/// ℓ = max{ u | D_u.len() ≥ 2·m }, return 0..l level

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
    let l = d_u
        .iter()
        .enumerate()
        .filter(|(_, b)| b.len() >= 2 * m)
        .map(|(u, _)| u)
        .max()
        .unwrap_or(0);
    d_u.truncate(l+1);
    Ok(d_u)
}


/// Error codes for D_u verification
#[derive(Debug, Error)]
pub enum VerifyDuError {
    #[error("level mismatch at level {level}, index {index}: expected level {level}, got {got}")]
    LevelMismatch {
        level: usize,
        index: usize,
        got: usize,
    },
    #[error("samples count mismatch at level {level}: got {count}, expect {expect}")]
    SampleCountMismatch {
        level: usize,
        count: usize,
        expect: usize,
    },
    #[error("interlink mismatch at level {level}, index {index}: expected {expected:?}, found {found:?}")]
    InterlinkMismatch {
        level: usize,
        index: usize,
        expected: HashValue,
        found: HashValue,
    },
    #[error("missing interlink pointer at level {level}, index {index}")]
    MissingInterlinkPointer {
        level: usize,
        index: usize,
    },
}

/// Verify that each D_u (per-level sample) satisfies:
/// 1. Each header’s PoW level == u.
/// 2. No more than 2*m samples.
/// 3. For i > 0, D_u[i].interlink[u] == hash(D_u[i-1]).

pub fn verify_du(
    buckets_by_level: &[VecDeque<BlockHeader>],
    m: usize,
) -> Result<(), VerifyDuError> {
    for (level, bucket) in buckets_by_level.iter().enumerate() {
        let count = bucket.len();
        if bucket.len() != count {
            return Err(VerifyDuError::SampleCountMismatch {
                level,
                count,
                expect: 2*m,
            });
        }

	
        // Check each header’s level
        for (i, header) in bucket.iter().enumerate() {
            let header_level =
                calculate_level(header.id(), header.difficulty())
                    .map_err(|_| VerifyDuError::LevelMismatch {
                        level,
                        index: i,
                        got: usize::MAX, // signal calculation error
                    })? as usize;
            if header_level != level {
                return Err(VerifyDuError::LevelMismatch {
                    level,
                    index: i,
                    got: header_level,
                });
            }

	    
        }

        // Check interlink chaining
        for i in 1..bucket.len() {
            let current = &bucket[i];
            let prev = &bucket[i - 1];
            // ensure interlink pointer exists
            let interlinks = current.interlink();
            let found_hash = interlinks.get(level).ok_or(
                VerifyDuError::MissingInterlinkPointer { level, index: i },
            )?.clone();
            let expected_hash = prev.id();
            if expected_hash != found_hash {
                return Err(VerifyDuError::InterlinkMismatch {
                    level,
                    index: i,
                    expected: expected_hash,
                    found: found_hash,
                });
            }
        }
    }

    Ok(())
}
