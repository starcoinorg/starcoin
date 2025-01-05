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
    use std::string;
    use std::vector;

    use starcoin_framework::smt_tree_hasher;
    use starcoin_framework::smt_utils;
    use starcoin_std::debug;

    #[test_only]
    use starcoin_framework::smt_hash;

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
        expect_root_hash: &vector<u8>,
        sibling_nodes: &vector<vector<u8>>,
        leaf_path: &vector<u8>,
        leaf_value_hash: &vector<u8>
    ): bool {
        debug::print(
            &string::utf8(b"smt_proofs::verify_membership_proof | entered, leaf path & leaf value hash & sibling_nodes")
        );
        debug::print(leaf_path);
        debug::print(leaf_value_hash);
        debug::print(sibling_nodes);

        let (leaf_hash, leaf_value) = smt_tree_hasher::digest_leaf(leaf_path, leaf_value_hash);
        debug::print(
            &string::utf8(
                b"smt_proofs::verify_membership_proof | after smt_tree_hasher::digest_leaf, leaf_path & leaf_value: "
            )
        );
        debug::print(&leaf_hash);
        debug::print(&leaf_value);

        let ret_hash = compute_root_hash(leaf_path, &leaf_hash, sibling_nodes);
        debug::print(
            &string::utf8(
                b"smt_proofs::verify_membership_proof | after Self::compute_root_hash, ret_hash & expect_root_hash: "
            )
        );
        debug::print(&ret_hash);
        debug::print(expect_root_hash);
        ret_hash == *expect_root_hash
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
    public fun compute_root_hash(
        path: &vector<u8>,
        node_hash: &vector<u8>,
        side_nodes: &vector<vector<u8>>
    ): vector<u8> {
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


    // This function mainly verifies that the value of the 0x1::chain_id::ChainId structure is 0xff (i.e. 255)
    // after the genesis of the test network is started.
    //
    // The test location of the rust code function name is `test_get_chain_id_after_genesis_with_proof_verify`
    //
    // expected_root_hash: HashValue(0xf65860f575bf2a198c069adb4e7872037e3a329b63ef617e40afa39b87b067c8),
    // element_key: HashValue(0x4cc8bd9df94b37c233555d9a3bba0a712c3c709f047486d1e624b2bcd3b83266),
    // element_blob: Some(Blob { Raw: 0xff  }),
    // siblings: [
    //   HashValue(0xcfb1462d4fc72f736eab2a56b2bf72ca6ad1c4e8c79557046a8b0adce047f007),
    //   HashValue(0x5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000),
    //   HashValue(0x5ca9febe74c7fde3fdcf2bd464de6d8899a0a13d464893aada2714c6fa774f9d),
    //   HashValue(0x1519a398fed69687cabf51adf831f0ee1650aaf79775d00135fc70f55a73e151),
    //   HashValue(0x50ce5c38983ba2eb196acd44e0aaedf040b1437ad1106e05ca452d7e27e4e03f),
    //   HashValue(0x55ed28435637a061a6dd9e20b72849199cd36184570f976b7e306a27bebf2fdf),
    //   HashValue(0x0dc23e31614798a6f67659b0b808b3eadc3b13a2a7bc03580a9e3004e45c2e6c),
    //   HashValue(0x83bed048bc0bc452c98cb0e9f1cc0f691919eaf756864fc44940c2d1e01da92a)
    // ]

    #[test]
    public fun test_verify_membership_proof() {
        let siblings = vector::empty<vector<u8>>();
        vector::push_back(&mut siblings, x"cfb1462d4fc72f736eab2a56b2bf72ca6ad1c4e8c79557046a8b0adce047f007");
        vector::push_back(&mut siblings, x"5350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000");
        vector::push_back(&mut siblings, x"5ca9febe74c7fde3fdcf2bd464de6d8899a0a13d464893aada2714c6fa774f9d");
        vector::push_back(&mut siblings, x"1519a398fed69687cabf51adf831f0ee1650aaf79775d00135fc70f55a73e151");
        vector::push_back(&mut siblings, x"50ce5c38983ba2eb196acd44e0aaedf040b1437ad1106e05ca452d7e27e4e03f");
        vector::push_back(&mut siblings, x"55ed28435637a061a6dd9e20b72849199cd36184570f976b7e306a27bebf2fdf");
        vector::push_back(&mut siblings, x"0dc23e31614798a6f67659b0b808b3eadc3b13a2a7bc03580a9e3004e45c2e6c");
        vector::push_back(&mut siblings, x"83bed048bc0bc452c98cb0e9f1cc0f691919eaf756864fc44940c2d1e01da92a");

        let expect_root_hash = x"f65860f575bf2a198c069adb4e7872037e3a329b63ef617e40afa39b87b067c8";
        let element_key = x"4cc8bd9df94b37c233555d9a3bba0a712c3c709f047486d1e624b2bcd3b83266";
        assert!(Self::verify_membership_proof(
            &expect_root_hash,
            &siblings,
            &element_key,
            &x"ff",
        ), 1110);

        // let node_data = vector::empty<u8>();
        // vector::append(&mut node_data, element_key);
        //
        // let value_hash = smt_hash::hash(&x"00000000000000ff");
        // debug::print(&string::utf8(b"test_verify_membership_proof | value_hash "));
        // debug::print(&value_hash);
        // vector::append(&mut node_data, value_hash);
        //
        // let node_hash = smt_hash::hash(&node_data);
        // debug::print(&string::utf8(b"test_verify_membership_proof | current_hash "));
        // debug::print(&node_hash);
        //
        // let actual_root = Self::compute_root_hash(
        //     &element_key,
        //     //&node_hash,
        //     &x"796c380bdad1231f930708197d9d4ddffe61e8bf2b3d817a0efe21230b11ae2e",
        //     &siblings,
        // );
        // assert!(actual_root == expect_root_hash, 1110);

    }
}

