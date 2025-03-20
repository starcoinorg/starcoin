address StarcoinFramework {
/// Utility for converting a Move value to its binary representation in BCS (Diem Canonical
/// Serialization). BCS is the binary encoding for Move resources and other non-module values
/// published on-chain.
module BCS {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }
    /// Return the binary representation of `v` in BCS (Starcoin Canonical Serialization) format
    native public fun to_bytes<MoveValue: store>(v: &MoveValue): vector<u8>;

    /// Return the address of key bytes
    native public fun to_address(key_bytes: vector<u8>): address;
    // ------------------------------------------------------------------------
    // Specification
    // ------------------------------------------------------------------------


    spec native fun serialize<MoveValue>(v: &MoveValue): vector<u8>;
}
}
