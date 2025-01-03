module starcoin_framework::smt_hash {
    use std::error;
    use std::vector;

    use starcoin_framework::hash;
    use starcoin_framework::smt_utils;

    const SIZE_ZERO_BYTES: vector<u8> = x"0000000000000000000000000000000000000000000000000000000000000000";

    public fun size(): u64 {
        32
    }

    public fun hash(data: &vector<u8>): vector<u8> {
        hash::sha3_256(*data)
    }

    public fun size_zero_bytes(): vector<u8> {
        SIZE_ZERO_BYTES
    }

    const ERROR_INVALID_PATH_BYTES_LENGTH: u64 = 101;
    const ERROR_INVALID_PATH_BITS_LENGTH: u64 = 102;
    const ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH: u64 = 103;
    //const NODE_DATA_LENGTH: u64 = 32;


    public fun path_bits_to_bool_vector_from_msb(path: &vector<u8>): vector<bool> {
        let path_len = vector::length<u8>(path);
        assert!(path_len == Self::size(), error::invalid_argument(ERROR_INVALID_PATH_BYTES_LENGTH));
        let result_vec = smt_utils::bits_to_bool_vector_from_msb(path);
        assert!(
            vector::length<bool>(&result_vec) == Self::size() * 8,// smt_tree_hasher::path_size_in_bits(),
            error::invalid_state(ERROR_INVALID_PATH_BITS_LENGTH)
        );
        result_vec
    }


    // Split sibling nodes data from concatenated data.
    // Due `Move` API call not yet support the parameter type such as vector<vector<u8>>,
    // so we concat all vectors into one vector<u8>.
    public fun split_side_nodes_data(side_nodes_data: &vector<u8>): vector<vector<u8>> {
        let node_data_length = Self::size();
        let len = vector::length(side_nodes_data);
        assert!(len % node_data_length == 0, error::invalid_state(ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH));

        if (len > 0) {
            let result = vector::empty<vector<u8>>();
            let size = len / node_data_length;
            let idx = 0;
            while (idx < size) {
                let start = idx * node_data_length;
                let end = start + node_data_length;
                vector::push_back(&mut result, smt_utils::sub_u8_vector(side_nodes_data, start, end));
                idx = idx + 1;
            };
            result
        } else {
            vector::empty<vector<u8>>()
        }
    }
}