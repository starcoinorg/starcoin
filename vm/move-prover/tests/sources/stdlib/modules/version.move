address 0x0 {

module Version {
    use 0x0::Config;
    use 0x0::Signer;
    use 0x0::Transaction;

    struct T {
        major: u64,
    }

    public fun initialize(account: &signer) {
        Transaction::assert(Signer::address_of(account) == Config::default_config_address(), 1);

        Config::publish_new_config<Self::T>(
            account,
            T { major: 1 },
        );
    }

    public fun set(account: &signer, major: u64) {
        let old_config = Config::get<Self::T>();

        Transaction::assert(
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
