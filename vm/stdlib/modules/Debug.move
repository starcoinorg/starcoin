address 0x1 {

module Debug {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    native public fun print<T>(x: &T);

    native public fun print_stack_trace();
}

}
