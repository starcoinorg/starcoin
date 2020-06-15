address 0x1 {

module Version {
    use 0x1::Config;
    use 0x1::Signer;


    struct T {
        major: u64,
    }

    public fun initialize(account: &signer) {
        assert(Signer::address_of(account) == Config::default_config_address(), 1);

        Config::publish_new_config<Self::T>(
            account,
            T { major: 1 },
        );
    }

    public fun set(account: &signer, major: u64) {
        let old_config = Config::get<Self::T>();

        assert(
            old_config.major < major,
            25
        );

        Config::set<Self::T>(
            account,
            T { major }
        );
    }
}

}
