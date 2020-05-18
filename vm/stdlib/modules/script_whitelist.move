address 0x0{

module ScriptWhitelist {
    use 0x0::Config;

    struct T { payload: vector<u8> }

    public fun initialize(payload: vector<u8>) {
        Config::publish_new_config<Self::T>(T { payload })
    }

    public fun set(payload: vector<u8>) {
        Config::set<Self::T>(T { payload } )
    }
}
}