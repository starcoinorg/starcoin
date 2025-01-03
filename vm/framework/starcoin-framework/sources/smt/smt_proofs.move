// Sparse Merkle Tree proof for non-membership,
// reference Starcoin project's source file located at: "commons/forkable-jellyfish-merkle/src/proof.rs"
//
// Computes the hash of internal node according to [`JellyfishTree`](crate::JellyfishTree)
// data structure in the logical view. `start` and `nibble_height` determine a subtree whose
// root hash we want to get. For an internal node with 16 children at the bottom level, we compute
// the root hash of it as if a full binary Merkle tree with 16 leaves as below:
//
// ```text
//   4 ->              +------ root hash ------+
//                     |                       |
//   3 ->        +---- # ----+           +---- # ----+
//               |           |           |           |
//   2 ->        #           #           #           #
//             /   \       /   \       /   \       /   \
//   1 ->     #     #     #     #     #     #     #     #
//           / \   / \   / \   / \   / \   / \   / \   / \
//   0 ->   0   1 2   3 4   5 6   7 8   9 A   B C   D E   F
//   ^
// height
// ```
//
// As illustrated above, at nibble height 0, `0..F` in hex denote 16 chidren hashes.  Each `#`
// means the hash of its two direct children, which will be used to generate the hash of its
// parent with the hash of its sibling. Finally, we can get the hash of this internal node.
//
// However, if an internal node doesn't have all 16 chidren exist at height 0 but just a few of
// them, we have a modified hashing rule on top of what is stated above:
// 1. From top to bottom, a node will be replaced by a leaf child if the subtree rooted at this
// node has only one child at height 0 and it is a leaf child.
// 2. From top to bottom, a node will be replaced by the placeholder node if the subtree rooted at
// this node doesn't have any child at height 0. For example, if an internal node has 3 leaf
// children at index 0, 3, 8, respectively, and 1 internal node at index C, then the computation
// graph will be like:
//
// ```text
//   4 ->              +------ root hash ------+
//                     |                       |
//   3 ->        +---- # ----+           +---- # ----+
//               |           |           |           |
//   2 ->        #           @           8           #
//             /   \                               /   \
//   1 ->     0     3                             #     @
//                                               / \
//   0 ->                                       C   @
//   ^
// height
// Note: @ denotes placeholder hash.
// ```
module starcoin_framework::smt_proofs {

    use std::error;
    use std::vector;

    use starcoin_framework::smt_tree_hasher;
    use starcoin_framework::smt_utils;
    use starcoin_std::debug;

    const ERROR_KEY_ALREADY_EXISTS_IN_PROOF: u64 = 101;
    const ERROR_COUNT_COMMON_PREFIX: u64 = 102;
    const BIT_RIGHT: bool = true;

    public fun verify_non_membership_proof_by_key(
        root_hash: &vector<u8>,
        non_membership_leaf_data: &vector<u8>,
        side_nodes: &vector<vector<u8>>,
        key: &vector<u8>
    ): bool {
        let leaf_path = smt_tree_hasher::digest(key);
        verify_non_membership_proof_by_leaf_path(root_hash, non_membership_leaf_data, side_nodes, &leaf_path)
    }

    // Verify non-membership proof by leaf path.
    // Return true if leaf path(key) is not in the tree.
    public fun verify_non_membership_proof_by_leaf_path(
        root_hash: &vector<u8>,
        non_membership_leaf_data: &vector<u8>,
        side_nodes: &vector<vector<u8>>,
        leaf_path: &vector<u8>
    ): bool {
        let non_membership_leaf_hash = if (vector::length<u8>(non_membership_leaf_data) > 0) {
            let (non_membership_leaf_path, _) = smt_tree_hasher::parse_leaf(non_membership_leaf_data);
            assert!(*leaf_path != *&non_membership_leaf_path, error::invalid_state(ERROR_KEY_ALREADY_EXISTS_IN_PROOF));
            assert!(
                (smt_utils::count_common_prefix(leaf_path, &non_membership_leaf_path) >= vector::length(side_nodes)),
                ERROR_COUNT_COMMON_PREFIX
            );
            smt_tree_hasher::digest_leaf_data(non_membership_leaf_data)
        } else {
            smt_tree_hasher::placeholder()
        };
        compute_root_hash(leaf_path, &non_membership_leaf_hash, side_nodes) == *root_hash
    }

    public fun verify_membership_proof_by_key_value(
        root_hash: &vector<u8>,
        side_nodes: &vector<vector<u8>>,
        key: &vector<u8>,
        value: &vector<u8>,
        is_raw_value: bool
    ): bool {
        let leaf_path = smt_tree_hasher::digest(key);
        let leaf_value_hash = if (is_raw_value) {
            &smt_tree_hasher::digest(value)
        } else {
            value
        };
        verify_membership_proof(root_hash, side_nodes, &leaf_path, leaf_value_hash)
    }

    public fun verify_membership_proof(
        root_hash: &vector<u8>,
        side_nodes: &vector<vector<u8>>,
        leaf_path: &vector<u8>,
        leaf_value_hash: &vector<u8>
    ): bool {
        let (leaf_hash, _) = smt_tree_hasher::digest_leaf(leaf_path, leaf_value_hash);
        compute_root_hash(leaf_path, &leaf_hash, side_nodes) == *root_hash
    }

    public fun compute_root_hash_by_leaf(
        leaf_path: &vector<u8>,
        leaf_value_hash: &vector<u8>,
        side_nodes: &vector<vector<u8>>
    ): vector<u8> {
        let (leaf_hash, _) = smt_tree_hasher::digest_leaf(leaf_path, leaf_value_hash);
        compute_root_hash(leaf_path, &leaf_hash, side_nodes)
    }

    // Compute root hash after a new leaf included.
    public fun compute_root_hash_new_leaf_included(
        leaf_path: &vector<u8>,
        leaf_value_hash: &vector<u8>,
        non_membership_leaf_data: &vector<u8>,
        side_nodes: &vector<vector<u8>>
    ): vector<u8> {
        let (new_side_nodes, leaf_node_hash) = create_membership_side_nodes(
            leaf_path,
            leaf_value_hash,
            non_membership_leaf_data,
            side_nodes
        );

        compute_root_hash(leaf_path, &leaf_node_hash, &new_side_nodes)
    }

    // Create membership proof from non-membership proof.
    // Return root hash, side nodes.
    public fun create_membership_proof(
        leaf_path: &vector<u8>,
        leaf_value_hash: &vector<u8>,
        non_membership_leaf_data: &vector<u8>,
        side_nodes: &vector<vector<u8>>
    ): (vector<u8>, vector<vector<u8>>) {
        let (new_side_nodes, leaf_node_hash) = create_membership_side_nodes(
            leaf_path,
            leaf_value_hash,
            non_membership_leaf_data,
            side_nodes
        );
        let new_root_hash = compute_root_hash(leaf_path, &leaf_node_hash, &new_side_nodes);
        (new_root_hash, new_side_nodes)
    }

    // Create membership proof side nodes from non-membership proof.
    fun create_membership_side_nodes(
        leaf_path: &vector<u8>,
        leaf_value_hash: &vector<u8>,
        non_membership_leaf_data: &vector<u8>,
        side_nodes: &vector<vector<u8>>
    ): (vector<vector<u8>>, vector<u8>) {
        let side_nodes_len = vector::length<vector<u8>>(side_nodes);
        let (new_leaf_hash, _) = smt_tree_hasher::digest_leaf(leaf_path, leaf_value_hash);
        let new_side_nodes = if (vector::length(non_membership_leaf_data) > 0) {
            let (non_membership_leaf_path, _) = smt_tree_hasher::parse_leaf(non_membership_leaf_data);
            assert!(*leaf_path != *&non_membership_leaf_path, error::invalid_state(ERROR_KEY_ALREADY_EXISTS_IN_PROOF));

            let common_prefix_count = smt_utils::count_common_prefix(leaf_path, &non_membership_leaf_path);
            let old_leaf_hash = smt_tree_hasher::digest_leaf_data(non_membership_leaf_data);
            let new_side_nodes = vector::empty<vector<u8>>();

            vector::push_back(&mut new_side_nodes, old_leaf_hash);
            if (common_prefix_count > side_nodes_len) {
                let place_holder_len = (common_prefix_count - side_nodes_len);
                // Put placeholders
                let idx = 0;
                while (idx < place_holder_len) {
                    vector::push_back(&mut new_side_nodes, smt_tree_hasher::placeholder());
                    idx = idx + 1;
                };
            };
            new_side_nodes
        } else {
            vector::empty<vector<u8>>()
        };

        // Push old siblings into the new siblings array
        let idx = 0;
        while (idx < side_nodes_len) {
            vector::push_back(&mut new_side_nodes, *vector::borrow(side_nodes, idx));
            idx = idx + 1;
        };
        (new_side_nodes, new_leaf_hash)
    }

    // Compute root hash.
    // The parameter `node_hash` is leaf or internal node hash.
    fun compute_root_hash(
        path: &vector<u8>,
        node_hash: &vector<u8>,
        side_nodes: &vector<vector<u8>>
    ): vector<u8> {
        debug::print(side_nodes);
        let side_nodes_len = vector::length<vector<u8>>(side_nodes);

        let i = 0;
        let current_hash = *node_hash;
        while (i < side_nodes_len) {
            let bit = smt_utils::get_bit_at_from_msb(path, side_nodes_len - i - 1);
            let sibling_hash = vector::borrow<vector<u8>>(side_nodes, i);
            if (bit == BIT_RIGHT) {
                (current_hash, _) = smt_tree_hasher::digest_node(sibling_hash, &current_hash);
            } else {
                // left
                (current_hash, _) = smt_tree_hasher::digest_node(&current_hash, sibling_hash);
            };
            i = i + 1;
        };
        current_hash
    }

    //    struct SparseMerkleInternalNode has store, drop {
    //        left_child: vector<u8>,
    //        right_child: vector<u8>,
    //    }

    //    struct SparseMerkleLeafNode has store, drop {
    //        key: vector<u8>,
    //    }
}

#[test_only]
module starcoin_framework::smt_non_membership_proof_test {

    use std::hash;
    use std::vector;

    use starcoin_framework::smt_hash;
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

        let bit_vec = smt_hash::path_bits_to_bool_vector_from_msb(&hash);
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
        let bits1 = smt_hash::path_bits_to_bool_vector_from_msb(
            &x"0000000000000000000000000000000000000000000000000000000000000000"
        );
        let bits2 = smt_hash::path_bits_to_bool_vector_from_msb(
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