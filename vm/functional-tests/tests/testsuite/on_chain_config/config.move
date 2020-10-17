// Test Config
//! account: alice
//! account: bob

//! sender: alice
module MyConfig{
    use 0x1::Config;
    use 0x1::Signer;

    struct MyConfig{
        version: u64,
    }

    resource struct CapHolder{
        cap: Config::ModifyConfigCapability<MyConfig>,
    }

    public fun new_config(version: u64): MyConfig{
        MyConfig{
            version: version,
        }
    }

    public fun init(account: &signer){
        assert(Signer::address_of(account) == {{alice}}, 1000);
        Config::publish_new_config<MyConfig>(account, MyConfig{
            version: 0,
        });
    }

    public fun publish_new_config_with_capability(account: &signer, myconfig: MyConfig){
        assert(Signer::address_of(account) == {{bob}}, 1000);
        let cap = Config::publish_new_config_with_capability<MyConfig>(account, myconfig);
        move_to(account, CapHolder{cap: cap});
    }

    public fun extract_modify_config_capability(account: &signer){
        assert(Signer::address_of(account) == {{alice}}, 1000);
        let cap = Config::extract_modify_config_capability<MyConfig>(account);
        move_to(account, CapHolder{cap: cap});
    }

    public fun restore_modify_config_capability() acquires CapHolder{
        let CapHolder{cap:cap} = move_from<CapHolder>({{alice}});
        Config::restore_modify_config_capability(cap);
    }

    public fun update_my_config(version: u64) acquires CapHolder{
        let holder = borrow_global_mut<CapHolder>({{alice}});
        Config::set_with_capability(&mut holder.cap, MyConfig{version:version});
    }

    public fun get_my_config(): u64 {
        Config::get_by_address<MyConfig>({{alice}}).version
    }

    public fun get(): u64 {
        Config::get_by_address<MyConfig>({{alice}}).version
    }

}

// check: EXECUTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::MyConfig;

fun main(account: &signer) {
    MyConfig::init(account);
}
}

// check: EXECUTED

//! new-transaction
//! sender: bob
script {
use {{alice}}::MyConfig;

fun main(account: &signer) {
    MyConfig::publish_new_config_with_capability(account,MyConfig::new_config(10));
}
}

// check: EXECUTED

// update config by Config module
//! new-transaction
//! sender: alice
script {
use 0x1::Config;
use {{alice}}::MyConfig;

fun main(account: &signer) {
    Config::set(account, MyConfig::new_config(2));
    assert(MyConfig::get_my_config() == 2, 1001);
     assert(MyConfig::get() == 2, 1002);
}
}

// check: EXECUTED

// extract modify capability
//! new-transaction
//! sender: alice
script {
use {{alice}}::MyConfig;

fun main(account: &signer) {
    MyConfig::extract_modify_config_capability(account);
}
}

// check: EXECUTED

// update config by Config module fail, no capability.
//! new-transaction
//! sender: alice
script {
use 0x1::Config;
use {{alice}}::MyConfig;

fun main(account: &signer) {
    Config::set(account, MyConfig::new_config(3));
    assert(MyConfig::get_my_config() == 3, 1002);
//    assert(MyConfig::get() == 3, 1003);
}
}

// check: ABORTED
// check: 25860


// Any one can update config by MyConfig module.
//! new-transaction
//! sender: bob
script {
use {{alice}}::MyConfig;

fun main() {
    MyConfig::update_my_config(4);
    assert(MyConfig::get_my_config() == 4, 1003);
    assert(MyConfig::get() == 4, 1004);
}
}

// check: EXECUTED

// restore modify capability
//! new-transaction
//! sender: bob
script {
use {{alice}}::MyConfig;

fun main() {
    MyConfig::restore_modify_config_capability();
}
}

// check: EXECUTED

// alice can update config by Config module after restore capability.
//! new-transaction
//! sender: alice
script {
use 0x1::Config;
use {{alice}}::MyConfig;

fun main(account: &signer) {
    Config::set(account, MyConfig::new_config(5));
    assert(MyConfig::get_my_config() == 5, 1004);
    assert(MyConfig::get() == 5, 1005);
}
}

// check: EXECUTED
