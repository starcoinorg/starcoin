pub mod interlink;
mod proof;

//consensus const

/// k>= finality blocks
const K_TAIL_BLOCKS: usize = 600;
/// m >= 3k
const M_PER_LEVEL: usize = 1800;
