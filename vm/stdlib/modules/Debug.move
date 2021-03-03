address 0x1 {
/// The module provide debug print for Move.
module Debug {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// Print data of Type `T`.
    native public fun print<T>(x: &T);

    /// Print current stack.
    native public fun print_stack_trace();
}

}
