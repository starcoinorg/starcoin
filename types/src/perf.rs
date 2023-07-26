//!
//! A module for performance critical constants which depend on consensus parameters.
//! The constants in this module should all be revisited if mainnet consensus parameters change.
//!

/// The default target depth for reachability reindexes.
pub const DEFAULT_REINDEX_DEPTH: u64 = 100;

/// The default slack interval used by the reachability
/// algorithm to encounter for blocks out of the selected chain.
pub const DEFAULT_REINDEX_SLACK: u64 = 1 << 12;

#[derive(Clone, Debug)]
pub struct PerfParams {
    //
    // Cache sizes
    //
    /// Preferred cache size for header-related data
    pub header_data_cache_size: u64,

    /// Preferred cache size for block-body-related data which
    /// is typically orders-of magnitude larger than header data
    /// (Note this cannot be set to high due to severe memory consumption)
    pub block_data_cache_size: u64,

    /// Preferred cache size for UTXO-related data
    pub utxo_set_cache_size: u64,

    /// Preferred cache size for block-window-related data
    pub block_window_cache_size: u64,

    //
    // Thread-pools
    //
    /// Defaults to 0 which indicates using system default
    /// which is typically the number of logical CPU cores
    pub block_processors_num_threads: usize,

    /// Defaults to 0 which indicates using system default
    /// which is typically the number of logical CPU cores
    pub virtual_processor_num_threads: usize,
}

pub const PERF_PARAMS: PerfParams = PerfParams {
    header_data_cache_size: 10_000,
    block_data_cache_size: 200,
    utxo_set_cache_size: 10_000,
    block_window_cache_size: 2000,
    block_processors_num_threads: 0,
    virtual_processor_num_threads: 0,
};
