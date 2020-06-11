address 0x0 {
module Generic {
    /// Return module address, module name, and type name of `E`.
    native public fun type_of<E>(): (address, vector<u8>, vector<u8>);
}
}