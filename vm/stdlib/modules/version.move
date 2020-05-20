address 0x0 {

module Version {
    use 0x0::Config;
    use 0x0::Transaction;

    struct T {
        major: u64,
    }

    public fun initialize() {
        Transaction::assert(Transaction::sender() == Config::default_account_config::config_address(), 1);

        Config::publish_new_config<Self::T>(
            T { major: 1 },
        );
    }

    public fun set(major: u64) {
        let old_config = Config::get<Self::T>();

        Transaction::assert(
            old_config.major < major,
            25
        );

        Config::set<Self::T>(
            T { major }
        );
    }
}

}
