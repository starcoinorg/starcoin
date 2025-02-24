module starcoin_framework::starcoin_proof_bit {
    use std::vector;

    public fun get_bit(data: &vector<u8>, index: u64): bool {
        let pos = index / 8;
        let bit = (7 - index % 8);
        (*vector::borrow(data, pos) >> (bit as u8)) & 1u8 != 0
    }
}