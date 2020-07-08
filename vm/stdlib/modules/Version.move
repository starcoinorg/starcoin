address 0x1 {

module Version {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;


    struct Version {
        major: u64,
    }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

        Config::publish_new_config<Self::Version>(
            account,
            Version { major: 1 },
        );
    }

    public fun set(account: &signer, major: u64) {
        let old_config = Config::get<Self::Version>();

        assert(
            old_config.major < major,
            25
        );

        Config::set<Self::Version>(
            account,
            Version { major }
        );
    }
}

}
