use anyhow::Result;
use starcoin_consensus::difficult_to_target;
use starcoin_crypto::HashValue;
use starcoin_types::{block::BlockHeader, U256};

pub const MAX_LEVELS: usize = 255;
/// Build the child-block interlink (variable length, genesis ⇒ empty vec).
///
/// MAX_LEVELS is the *theoretical* upper bound (255 for SHA-256).
pub fn calculate_interlink(parent: &BlockHeader) -> Vec<HashValue> {
    // Derive μ(parent)
    let mut parent_level = calculate_level(parent.id(), parent.difficulty())
        .expect("level must be computable") as usize;

    // Clamp in case someone sets MAX_LEVELS < 255
    if parent_level >= MAX_LEVELS {
        parent_level = MAX_LEVELS - 1;
    }

    assert!(
        parent.interlink().len() >= parent_level + 1,
        "parent interlink too short"
    );
    assert!(
        parent.interlink().len() <= MAX_LEVELS,
        "parent interlink too long"
    );

    // determine exact final length
    let suffix_len = parent.interlink().len().saturating_sub(parent_level + 1);

    let final_len = parent_level + 1 + suffix_len;
    assert!(final_len <= MAX_LEVELS);

    let mut interlink = Vec::with_capacity(final_len);

    // prefix: copy parent.hash μ+1 times
    interlink.extend(std::iter::repeat(parent.id()).take(parent_level + 1));

    // suffix: copy untouched tail from parent
    interlink.extend(
        parent
            .interlink()
            .iter()
            .skip(parent_level + 1)
            .take(suffix_len)
            .copied(),
    );

    debug_assert_eq!(interlink.len(), final_len);
    interlink
}

fn verify_interlink(child: &BlockHeader, parent: &BlockHeader) -> Result<()> {
    assert!(child.interlink().len() <= MAX_LEVELS, "interlink too long");
    // ... other check
    Ok(())
}

/// Returns the NiPoPoW level µ of this block.
///
/// level 0 – “normal” PoW: hash ≤ target; every valid block is at least level 0  
/// level 1 – hash ≤ target / 2 (≈ 50 % of blocks); higher levels get one bit rarer each  
/// level 255 – theoretical maximum (hash = 1)
pub fn calculate_level(pow_hash: HashValue, difficulty: U256) -> Result<u8> {
    // µ = floor(log₂(target / hash ))

    let target: U256 = difficult_to_target(difficulty)?;
    let pow_hash_val: U256 = pow_hash.into();
    if pow_hash_val.is_zero() {
        return Ok(255);
    }
    // ratio = T / H
    let ratio = target
        .checked_div(pow_hash_val)
        .ok_or_else(|| anyhow::anyhow!("divide-by-zero in target/hash"))?;
    // µ = floor(log₂(ratio)) = ratio.bits()-1
    Ok(ratio.bits().saturating_sub(1) as u8)
}
