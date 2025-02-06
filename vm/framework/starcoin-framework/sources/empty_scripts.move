module starcoin_framework::empty_scripts {
    // A empty scripts module for call a script but do nothing.

    spec module {
        pragma verify = false;
        pragma aborts_if_is_partial = false;
        pragma aborts_if_is_strict = false;
    }

    public entry fun empty_script() {}
}