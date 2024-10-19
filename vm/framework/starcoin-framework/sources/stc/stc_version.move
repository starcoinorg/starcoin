/// `Version` tracks version of something, like current VM version.
module starcoin_framework::stc_version {
    use starcoin_framework::on_chain_config;

    const EMAJOR_TO_OLD: u64 = 101;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// Version.
    struct Version has copy, drop, store {
        /// major number.
        major: u64,
    }

    /// Create a new version.
    public fun new_version(major: u64): Version {
        Version { major }
    }

    spec new_version {
        aborts_if false;
    }

    /// Get version under `addr`.
    public fun get(addr: address): u64 {
        let version = on_chain_config::get_by_address<Self::Version>(addr);
        version.major
    }

    spec get {
        aborts_if !exists<on_chain_config::Config<Version>>(addr);
    }
}