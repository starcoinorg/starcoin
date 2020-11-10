address 0x1 {
module Version {
    use 0x1::Config;

    const EMAJOR_TO_OLD: u64 = 101;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    struct Version {
        major: u64,
    }

    public fun new_version(major: u64): Version {
        Version { major }
    }

    spec fun new_version {
        aborts_if false;
    }

    public fun get(addr: address): u64 {
        let version = Config::get_by_address<Self::Version>(addr);
        version.major
    }

    spec fun get {
        aborts_if !exists<Config::Config<Version>>(addr);
    }
}
}