//# init -n dev

//# faucet --addr alice

//# faucet --addr bob


//# publish
module alice::MyConfig {
    use starcoin_framework::on_chain_config;
    use starcoin_framework::signer;

    struct MyConfig has copy, drop, store {
        version: u64,
    }

    struct CapHolder has key, store {
        cap: on_chain_config::ModifyConfigCapability<MyConfig>,
    }

    public fun new_config(version: u64): MyConfig {
        MyConfig {
            version: version,
        }
    }

    public fun init(account: &signer) {
        assert!(signer::address_of(account) == @alice, 1000);
        on_chain_config::publish_new_config<MyConfig>(account, MyConfig {
            version: 0,
        });
    }

    public fun publish_new_config_with_capability(account: &signer, myconfig: MyConfig) {
        assert!(signer::address_of(account) == @bob, 1000);
        let cap = on_chain_config::publish_new_config_with_capability<MyConfig>(account, myconfig);
        move_to(account, CapHolder { cap: cap });
    }

    public fun extract_modify_config_capability(account: &signer) {
        assert!(signer::address_of(account) == @alice, 1000);
        let cap = on_chain_config::extract_modify_config_capability<MyConfig>(account);
        move_to(account, CapHolder { cap: cap });
    }

    public fun restore_modify_config_capability() acquires CapHolder {
        let CapHolder { cap: cap } = move_from<CapHolder>(@alice);
        on_chain_config::restore_modify_config_capability(cap);
    }

    public fun update_my_config(version: u64) acquires CapHolder {
        let holder = borrow_global_mut<CapHolder>(@alice);
        on_chain_config::set_with_capability(&mut holder.cap, MyConfig { version: version });
    }

    public fun get_my_config(): u64 {
        on_chain_config::get_by_address<MyConfig>(@alice).version
    }

    public fun get(): u64 {
        on_chain_config::get_by_address<MyConfig>(@alice).version
    }
}


//# run --signers alice
script {
    use alice::MyConfig;

    fun main(account: signer) {
        MyConfig::init(&account);
    }
}


//# run --signers bob
script {
    use alice::MyConfig;

    fun main(account: signer) {
        MyConfig::publish_new_config_with_capability(&account, MyConfig::new_config(10));
    }
}

//# run --signers alice
script {
    use starcoin_framework::on_chain_config;
    use alice::MyConfig;

    fun main(account: signer) {
        on_chain_config::set(&account, MyConfig::new_config(2));
        assert!(MyConfig::get_my_config() == 2, 1001);
        assert!(MyConfig::get() == 2, 1002);
    }
}


//# run --signers alice
script {
    use alice::MyConfig;

    fun main(account: signer) {
        MyConfig::extract_modify_config_capability(&account);
    }
}

//# run --signers alice
script {
    use starcoin_framework::on_chain_config;
    use alice::MyConfig;

    fun main(account: signer) {
        on_chain_config::set(&account, MyConfig::new_config(3));
        assert!(MyConfig::get_my_config() == 3, 1002);
        //    assert!(MyConfig::get() == 3, 1003);
    }
}

//# run --signers bob
script {
    use alice::MyConfig;

    fun main() {
        MyConfig::update_my_config(4);
        assert!(MyConfig::get_my_config() == 4, 1003);
        assert!(MyConfig::get() == 4, 1004);
    }
}


//# run --signers bob
script {
    use alice::MyConfig;

    fun main() {
        MyConfig::restore_modify_config_capability();
    }
}


//# run --signers alice
script {
    use starcoin_framework::on_chain_config;
    use alice::MyConfig;

    fun main(account: signer) {
        on_chain_config::set(&account, MyConfig::new_config(5));
        assert!(MyConfig::get_my_config() == 5, 1004);
        assert!(MyConfig::get() == 5, 1005);
    }
}
