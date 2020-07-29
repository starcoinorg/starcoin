address 0x1 {

module Version {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::ErrorCode;


    struct Version {
        major: u64,
    }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), ErrorCode::ENOT_GENESIS_ACCOUNT());

        Config::publish_new_config<Self::Version>(
            account,
            Version { major: 1 },
        );
    }

    public fun get():u64{
        let version = Config::get_by_address<Self::Version>(CoreAddresses::GENESIS_ACCOUNT());
        version.major
    }

    public fun set(account: &signer, major: u64) {
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), ErrorCode::ENOT_GENESIS_ACCOUNT());
        let old_config = Config::get<Self::Version>(account);

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
