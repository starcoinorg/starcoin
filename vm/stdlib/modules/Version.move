address 0x1 {
/// `Version` tracks version of something, like current VM version.
module Version {
    use 0x1::Config;

    const EMAJOR_TO_OLD: u64 = 101;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// Version.
    struct Version {
        /// major number.
        major: u64,
    }

    /// Create a new version.
    public fun new_version(major: u64): Version {
        Version { major }
    }

    spec fun new_version {
        aborts_if false;
    }

    /// Get version under `addr`.
    public fun get(addr: address): u64 {
        let version = Config::get_by_address<Self::Version>(addr);
        version.major
    }

    spec fun get {
        aborts_if !exists<Config::Config<Version>>(addr);
    }
}
}