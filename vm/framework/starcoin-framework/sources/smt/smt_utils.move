module starcoin_framework::smt_utils {

    use std::error;
    use std::vector;

    const ERROR_VECTORS_NOT_SAME_LENGTH: u64 = 103;
    const BIT_RIGHT: bool = true;
    const BIT_LEFT: bool = false;

    // Get the bit at an offset from the most significant bit.
    public fun get_bit_at_from_msb(data: &vector<u8>, position: u64): bool {
        let byte = (*vector::borrow<u8>(data, position / 8) as u64);
        // let bit = BitOperators::rshift(byte, ((7 - (position % 8)) as u8));
        let bit = byte >> ((7 - (position % 8)) as u8);
        if (bit & 1 != 0) {
            BIT_RIGHT
        } else {
            BIT_LEFT
        }
    }

    public fun count_common_prefix(data1: &vector<u8>, data2: &vector<u8>): u64 {
        let count = 0;
        let i = 0;
        while (i < vector::length(data1) * 8) {
            if (get_bit_at_from_msb(data1, i) == get_bit_at_from_msb(data2, i)) {
                count = count + 1;
            } else {
                break
            };
            i = i + 1;
        };
        count
    }

    public fun count_vector_common_prefix<ElementT: copy + drop>(
        vec1: &vector<ElementT>,
        vec2: &vector<ElementT>
    ): u64 {
        let vec_len = vector::length<ElementT>(vec1);
        assert!(vec_len == vector::length<ElementT>(vec2), error::invalid_state(ERROR_VECTORS_NOT_SAME_LENGTH));
        let idx = 0;
        while (idx < vec_len) {
            if (*vector::borrow(vec1, idx) != *vector::borrow(vec2, idx)) {
                break
            };
            idx = idx + 1;
        };
        idx
    }

    public fun bits_to_bool_vector_from_msb(data: &vector<u8>): vector<bool> {
        let i = 0;
        let vec = vector::empty<bool>();
        while (i < vector::length(data) * 8) {
            vector::push_back<bool>(&mut vec, get_bit_at_from_msb(data, i));
            i = i + 1;
        };
        vec
    }

    public fun concat_u8_vectors(v1: &vector<u8>, v2: vector<u8>): vector<u8> {
        let data = *v1;
        vector::append(&mut data, v2);
        data
    }

    public fun sub_u8_vector(vec: &vector<u8>, start: u64, end: u64): vector<u8> {
        let i = start;
        let result = vector::empty<u8>();
        let data_len = vector::length(vec);
        let actual_end = if (end < data_len) {
            end
        } else {
            data_len
        };
        while (i < actual_end) {
            vector::push_back(&mut result, *vector::borrow(vec, i));
            i = i + 1;
        };
        result
    }

    public fun sub_vector<ElementT: copy>(vec: &vector<ElementT>, start: u64, end: u64): vector<ElementT> {
        let i = start;
        let result = vector::empty<ElementT>();
        let data_len = vector::length(vec);
        let actual_end = if (end < data_len) {
            end
        } else {
            data_len
        };
        while (i < actual_end) {
            vector::push_back(&mut result, *vector::borrow(vec, i));
            i = i + 1;
        };
        result
    }
}