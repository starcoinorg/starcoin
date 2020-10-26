address 0x1 {
module Version {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Errors;

    const EMAJOR_TO_OLD: u64 = 101;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    struct Version {
        major: u64,
    }

    public fun initialize(account: &signer) {
        CoreAddresses::assert_genesis_address(account);
        Config::publish_new_config<Self::Version>(account, Version { major: 1 });
    }

    spec fun initialize {
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if
            exists<Config::ModifyConfigCapabilityHolder<Version>>(Signer::spec_address_of(account));
        aborts_if exists<Config::Config<Version>>(Signer::spec_address_of(account));
        ensures
            exists<Config::ModifyConfigCapabilityHolder<Version>>(Signer::spec_address_of(account));
        ensures exists<Config::Config<Version>>(Signer::spec_address_of(account));
    }

    public fun new_version(major: u64): Version {
        assert(Self::get() < major, 25);
        Version { major }
    }

    spec fun new_version {
        aborts_if !exists<Config::Config<Version>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if Self::get() >= major;
    }

    public fun get(): u64 {
        let version = Config::get_by_address<Self::Version>(CoreAddresses::GENESIS_ADDRESS());
        version.major
    }

    spec fun get {
        aborts_if !exists<Config::Config<Version>>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    public fun set(account: &signer, major: u64) {
        CoreAddresses::assert_genesis_address(account);
        let old_config = Config::get_by_address<Self::Version>(Signer::address_of(account));
        assert(old_config.major < major, Errors::invalid_argument(EMAJOR_TO_OLD));
        Config::set<Self::Version>(account, Version { major });
    }

    spec fun set {
        pragma verify = false;
        //Todo: data invariant does not hold
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if Config::spec_get<Version>(Signer::spec_address_of(account)).major >= major;
    }
}
}