/// The module for stdlib upgrade init scripts
///
module starcoin_framework::stdlib_upgrade_scripts {
    use starcoin_std::debug;

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = true;
    }

    public entry fun dummy_upgrade(
        sender: signer,
    ) {
        do_dummy_upgrade(&sender);
    }

    public fun do_dummy_upgrade(
        _sender: &signer,
    ) {
        debug::print(&std::string::utf8(b"do_dummy_upgrade"));
    }

}
