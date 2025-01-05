
#[test_only]
module starcoin_framework::smt_non_membership_proof_test {

    use std::hash;
    use std::vector;

    use starcoin_framework::smt_tree_hasher;
    use starcoin_framework::smt_utils;
    use starcoin_std::debug;

    const TEST_CHAIN_ID: u64 = 218;

    struct MerkleInternalNode has store, drop {
        left_child: vector<u8>,
        right_child: vector<u8>,
    }

    #[test]
    public fun test_iter_bits() {
        let hash = x"1000000000000000000000000000000000000000000000000000000000000000";
        debug::print(&hash::sha3_256(*&hash));

        let bit_vec = smt_utils::path_bits_to_bool_vector_from_msb(&hash);
        debug::print(&bit_vec);
        assert!(vector::length<bool>(&bit_vec) == 256, 1101);

        let sub_bits = vector::slice<bool>(&bit_vec, 252, 256);
        debug::print(&sub_bits);
        assert!(vector::length<bool>(&sub_bits) == 4, 1102);
    }

    // #[test]
    // public fun test_bit() {
    //     assert!(BitOperators::and(1, 2) == 0, 1103);
    //     assert!(BitOperators::and(1, 3) == 1, 1104);
    //     assert!(BitOperators::and(1, 16 >> 4) == 1, 1105);
    // }

    #[test]
    public fun test_print_fix_keyword() {
        let k1 = x"01";
        let k2 = b"helloworld";
        debug::print(&k1);
        debug::print(&k2);
        debug::print(&hash::sha3_256(k1));
        debug::print(&hash::sha3_256(k2));
    }


    #[test]
    public fun test_get_bit() {
        // Print origin hash
        let origin_hash = x"1000000000000000000000000000000000000000000000000000000000000001";
        debug::print(&origin_hash);

        // Expect first byte is 'F', which binary is 11111111
        let first_byte = *vector::borrow(&origin_hash, 0);
        debug::print(&first_byte);

        // let bit = BitOperators::and(BitOperators::rshift((first_byte as u64), 4), (1 as u64));
        let bit = (first_byte >> 4 & 1);
        debug::print(&bit);
        assert!((first_byte >> 4 & 1) == 1, 1106);

        let bit_hash = vector::empty();
        let i = 0;
        while (i < 256) {
            vector::push_back(&mut bit_hash, smt_utils::get_bit_at_from_msb(&origin_hash, i));
            i = i + 1;
        };
        debug::print(&bit_hash);

        // Test skip bit
        vector::reverse(&mut bit_hash);
        let skip_bits = vector::slice<bool>(&bit_hash, 252, 256);
        debug::print(&skip_bits);

        let skip_bits_1 = vector::slice<bool>(&bit_hash, 0, 1);
        debug::print(&skip_bits_1);
    }

    #[test]
    public fun test_fixed_leaf_node_data() {
        let data = x"0076d3bc41c9f588f7fcd0d5bf4718f8f84b1c41b20882703100b9eb9413807c012767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7";
        let expect = x"da3c17cfd8be129f09b61272f8afcf42bf5b77cf7e405f5aa20c30684a205488";

        let crypto_hash = smt_tree_hasher::digest_leaf_data(&data);

        debug::print(&crypto_hash);
        debug::print(&expect);
        assert!(crypto_hash == expect, 1107);
    }

    #[test]
    public fun test_fixed_internal_node_data() {
        let left = x"24a7e02bc5b39e8a4b7d2396d2e637632d0938944d16d571f0485168461f46eb";
        let right = x"42bfc776a76b35ca641ee761a5f4bc6ebf2d4e2441c517f8a8e085dec3ca443c";
        let expect = x"060aec78413605e993f9338255b661ac794a68729ffa50022aca72b01586a306";

        let (crypto_hash, _) = smt_tree_hasher::digest_node(&left, &right);

        debug::print(&crypto_hash);
        debug::print(&expect);

        assert!(crypto_hash == expect, 1108);
    }

    #[test]
    fun test_common_prefix_bits_len() {
        let bits1 = smt_utils::path_bits_to_bool_vector_from_msb(
            &x"0000000000000000000000000000000000000000000000000000000000000000"
        );
        let bits2 = smt_utils::path_bits_to_bool_vector_from_msb(
            &x"1000000000000000000000000000000000000000000000000000000000000000"
        );
        debug::print(&bits1);
        debug::print(&bits2);
        let len = smt_utils::count_vector_common_prefix<bool>(&bits1, &bits2);
        debug::print(&len);
        assert!(len == 3, 1109);
    }

    #[test]
    public fun test_fixed_split_leaf_node_data() {
        let data = x"0076d3bc41c9f588f7fcd0d5bf4718f8f84b1c41b20882703100b9eb9413807c012767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7";
        let (leaf_node_path, leaf_node_value) = smt_tree_hasher::parse_leaf(&data);
        //assert!(prefix == x"00", 1110);

        debug::print(&leaf_node_path);
        debug::print(&x"76d3bc41c9f588f7fcd0d5bf4718f8f84b1c41b20882703100b9eb9413807c01");
        assert!(leaf_node_path == x"76d3bc41c9f588f7fcd0d5bf4718f8f84b1c41b20882703100b9eb9413807c01", 1106);

        debug::print(&leaf_node_value);
        debug::print(&x"2767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7");
        assert!(leaf_node_value == x"2767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7", 1107);
    }
}