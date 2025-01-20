module starcoin_framework::empty_scripts {
    // A empty scripts module for call a script but do nothing.

    use std::option;
use std::string;
use starcoin_std::debug;
use starcoin_framework::object;
use starcoin_framework::starcoin_coin::STC;
use starcoin_framework::coin;
spec module {
        pragma verify = false;
        pragma aborts_if_is_partial = false;
        pragma aborts_if_is_strict = false;
    }

    public entry fun empty_script() {}


    public entry fun test_metadata(_account: &signer) {
        debug::print(&string::utf8(b"test_metadata | entered"));
        let metadata = coin::paired_metadata<STC>();
        assert!(option::is_some(&metadata), 10000);
        let metdata_obj = option::destroy_some(metadata);
        assert!(object::is_object(object::object_address(&metdata_obj)), 10001);
        debug::print(&string::utf8(b"test_metadata | exited"));

    }
}