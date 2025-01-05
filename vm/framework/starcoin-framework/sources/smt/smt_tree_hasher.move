module starcoin_framework::smt_tree_hasher {

    use std::error;
    use std::vector;

    use starcoin_framework::smt_hash;
    use starcoin_framework::smt_utils;

    // sparse merkle tree leaf(node) prefix.
    const LEAF_PREFIX: vector<u8> = x"00";
    // sparse merkle tree (internal) node prefix.
    const NODE_PREFIX: vector<u8> = x"01";

    // Leaf node data include: prefix + leaf_path + leaf_value_hash
    //const LEAF_DATA_LENGTH: u64 = 65;
    //const NODE_LEFT_RIGHT_DATA_LENGTH: u64 = 32;
    //const LEAF_PATH_LENGTH: u64 = 32;

    const ERROR_INVALID_LEAF_DATA: u64 = 102;
    const ERROR_INVALID_NODE_DATA: u64 = 103;
    const ERROR_INVALID_LEAF_DATA_LENGTH: u64 = 104;
    const ERROR_INVALID_NODE_DATA_LENGTH: u64 = 105;

    // Parse leaf data.
    // Return values:
    //     leaf node path.
    //     leaf node value.
    public fun parse_leaf(data: &vector<u8>): (vector<u8>, vector<u8>) {
        let data_len = vector::length(data);

        let prefix_len = vector::length(&LEAF_PREFIX);
        assert!(data_len >= prefix_len + path_size(), error::invalid_argument(ERROR_INVALID_LEAF_DATA));
        assert!(smt_utils::sub_u8_vector(data, 0, prefix_len) == LEAF_PREFIX, error::invalid_argument(ERROR_INVALID_LEAF_DATA));

        let start = 0;
        let end = prefix_len;
        _ = start;//let prefix = smt_utils::sub_u8_vector(data, start, end);

        start = end;
        end = start + path_size();
        let leaf_node_path = smt_utils::sub_u8_vector(data, start, end);

        start = end;
        end = vector::length(data);
        let leaf_node_value = smt_utils::sub_u8_vector(data, start, end);
        (leaf_node_path, leaf_node_value)
    }

    //    #[test]
    //    #[expected_failure]
    //    public fun test_parse_leaf_1() {
    //        let data = x"0189bd5770d361dfa0c06a8c1cf4d89ef194456ab5cf8fc55a9f6744aff0bfef812767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7";
    //        let (leaf_node_path, leaf_node_value) = parse_leaf(&data);
    //        assert!(leaf_node_path == x"89bd5770d361dfa0c06a8c1cf4d89ef194456ab5cf8fc55a9f6744aff0bfef81", 101);
    //        assert!(leaf_node_value == x"2767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7", 101);
    //    }
    //
    //    #[test]
    //    public fun test_parse_leaf_2() {
    //        let data = x"0089bd5770d361dfa0c06a8c1cf4d89ef194456ab5cf8fc55a9f6744aff0bfef812767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7";
    //        let (leaf_node_path, leaf_node_value) = parse_leaf(&data);
    //        assert!(leaf_node_path == x"89bd5770d361dfa0c06a8c1cf4d89ef194456ab5cf8fc55a9f6744aff0bfef81", 101);
    //        assert!(leaf_node_value == x"2767f15c8af2f2c7225d5273fdd683edc714110a987d1054697c348aed4e6cc7", 101);
    //    }

    public fun parse_node(data: &vector<u8>): (vector<u8>, vector<u8>) {
        let data_len = vector::length(data);
        let prefix_len = vector::length(&NODE_PREFIX);
        assert!(data_len == prefix_len + path_size() * 2, error::invalid_argument(ERROR_INVALID_NODE_DATA));
        assert!(smt_utils::sub_u8_vector(data, 0, prefix_len) == NODE_PREFIX, error::invalid_argument(ERROR_INVALID_NODE_DATA));

        let start = 0;
        let end = prefix_len;
        _ = start;//let prefix = smt_utils::sub_u8_vector(data, start, end);

        start = end;
        end = start + path_size();
        let left_data = smt_utils::sub_u8_vector(data, start, end);

        start = end;
        end = vector::length(data);
        let right_data = smt_utils::sub_u8_vector(data, start, end);
        (left_data, right_data)
    }

    public fun digest_leaf(path: &vector<u8>, leaf_value: &vector<u8>): (vector<u8>, vector<u8>) {
        let value = LEAF_PREFIX;
        value = smt_utils::concat_u8_vectors(&value, *path);
        value = smt_utils::concat_u8_vectors(&value, *leaf_value);
        (smt_hash::hash(&value), value)
    }

    public fun create_leaf_data(path: &vector<u8>, leaf_value: &vector<u8>): vector<u8> {
        let value = LEAF_PREFIX;
        value = smt_utils::concat_u8_vectors(&value, *path);
        value = smt_utils::concat_u8_vectors(&value, *leaf_value);
        value
    }

    // Digest leaf data. The parameter `data` includes leaf key and value.
    public fun digest_leaf_data(data: &vector<u8>): vector<u8> {
        let data_len = vector::length(data);
        let prefix_len = vector::length(&LEAF_PREFIX);
        assert!(data_len >= prefix_len + path_size(), error::invalid_state(ERROR_INVALID_LEAF_DATA_LENGTH));
        assert!(smt_utils::sub_u8_vector(data, 0, prefix_len) == LEAF_PREFIX, error::invalid_argument(ERROR_INVALID_LEAF_DATA));
        smt_hash::hash(data)
    }

    public fun digest_node(left_data: &vector<u8>, right_data: &vector<u8>): (vector<u8>, vector<u8>) {
        let node_left_right_data_length = smt_hash::size();
        assert!(vector::length(left_data) == node_left_right_data_length, error::invalid_state(ERROR_INVALID_NODE_DATA_LENGTH));
        assert!(vector::length(right_data) == node_left_right_data_length, error::invalid_state(ERROR_INVALID_NODE_DATA_LENGTH));

        let value = NODE_PREFIX;
        value = smt_utils::concat_u8_vectors(&value, *left_data);
        value = smt_utils::concat_u8_vectors(&value, *right_data);
        (smt_hash::hash(&value), value)
    }

    public fun path(key: &vector<u8>): vector<u8> {
        digest(key)
    }

    public fun digest(data: &vector<u8>): vector<u8> {
        smt_hash::hash(data)
    }

    public fun path_size(): u64 {
        smt_hash::size()
    }

    public fun path_size_in_bits(): u64 {
        smt_hash::size() * 8
    }

    public fun placeholder(): vector<u8> {
        smt_hash::size_zero_bytes()
    }
}