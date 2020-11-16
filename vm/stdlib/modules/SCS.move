// Utility for converting a Move value to its binary representation in SCS (Libra Canonical
// Serialization). SCS is the binary encoding for Move resources and other non-module values
// published on-chain.

address 0x1 {
module SCS {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }
    // Return the binary representation of `v` in SCS (Starcoin Canonical Serialization) format
    native public fun to_bytes<MoveValue>(v: &MoveValue): vector<u8>;

    // Return the address of key bytes
    native public fun to_address(key_bytes: vector<u8>): address;
    // ------------------------------------------------------------------------
    // Specification
    // ------------------------------------------------------------------------

    spec module {
        native define serialize<MoveValue>(v: &MoveValue): vector<u8>;
    }
}
}
