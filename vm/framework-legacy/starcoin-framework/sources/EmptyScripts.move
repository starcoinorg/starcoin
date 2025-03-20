address StarcoinFramework {
    // A empty scripts module for call a script but do nothing.
    module EmptyScripts {

        spec module {
            pragma verify = false;
            pragma aborts_if_is_partial = false;
            pragma aborts_if_is_strict = false;
        }

        public entry fun empty_script() {
        }
    }
}