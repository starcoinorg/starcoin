address 0x1 {

module Hash {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }
    native public fun sha2_256(data: vector<u8>): vector<u8>;
    native public fun sha3_256(data: vector<u8>): vector<u8>;
}

}
