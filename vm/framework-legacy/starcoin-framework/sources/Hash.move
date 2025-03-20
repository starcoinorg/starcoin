address StarcoinFramework {
/// The module provide sha-hash functionality for Move.
module Hash {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }
    native public fun sha2_256(data: vector<u8>): vector<u8>;
    native public fun sha3_256(data: vector<u8>): vector<u8>;
    native public fun keccak_256(data: vector<u8>): vector<u8>;
    native public fun ripemd160(data: vector<u8>): vector<u8>;
}

}
