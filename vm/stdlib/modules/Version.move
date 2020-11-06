address 0x1 {
module Version {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::Errors;

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

    public fun set(account: &signer, major: u64) {
        let old_config = Config::get_by_address<Self::Version>(Signer::address_of(account));
        assert(old_config.major < major, Errors::invalid_argument(EMAJOR_TO_OLD));
        Config::set<Self::Version>(account, Version { major });
    }

    spec fun set {
        pragma verify = false;
        //Todo: data invariant does not hold
        aborts_if Config::spec_get<Version>(Signer::spec_address_of(account)).major >= major;
    }
}
}