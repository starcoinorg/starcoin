module starcoin_framework::starcoin_proof_verifier {
    use std::hash;
    use std::vector;

    use starcoin_framework::starcoin_proof_bit;
    use starcoin_framework::starcoin_proof_structured_hash;

    struct StarcoinMerkle has key {
        merkle_root: vector<u8>,
    }

    struct Node has store, drop {
        hash1: vector<u8>,
        hash2: vector<u8>,
    }

    const HASH_LEN_IN_BIT: u64 = 32 * 8;
    const SPARSE_MERKLE_LEAF_NODE: vector<u8> = b"SparseMerkleLeafNode";
    const SPARSE_MERKLE_INTERNAL_NODE: vector<u8> = b"SparseMerkleInternalNode";

    public fun create(signer: &signer, merkle_root: vector<u8>) {
        let s = StarcoinMerkle {
            merkle_root
        };
        move_to(signer, s);
    }

    public fun verify_on(
        merkle_address: address,
        account_address: vector<u8>,
        account_state_root_hash: vector<u8>,
        proofs: vector<vector<u8>>
    ): bool
    acquires StarcoinMerkle {
        let merkle = borrow_global<StarcoinMerkle>(merkle_address);
        verify(*&merkle.merkle_root, account_address, account_state_root_hash, proofs)
    }

    public fun verify(
        expected_root: vector<u8>,
        account_address: vector<u8>,
        account_state_root_hash: vector<u8>,
        proofs: vector<vector<u8>>
    ): bool {
        Self::computer_root_hash(hash::sha3_256(account_address), account_state_root_hash, proofs) == expected_root
    }

    public fun computer_root_hash(
        element_key: vector<u8>,
        element_blob_hash: vector<u8>,
        proofs: vector<vector<u8>>
    ): vector<u8> {
        let leaf_node = Node { hash1: element_key, hash2: element_blob_hash };
        let current_hash = starcoin_proof_structured_hash::hash(SPARSE_MERKLE_LEAF_NODE, &leaf_node);
        let i = 0;
        let proof_length = vector::length(&proofs);
        while (i < proof_length) {
            let sibling = *vector::borrow(&proofs, i);
            let bit = starcoin_proof_bit::get_bit(&element_key, proof_length - i - 1);
            let internal_node = if (bit) {
                Node { hash1: sibling, hash2: current_hash }
            } else {
                Node { hash1: current_hash, hash2: sibling }
            };
            current_hash = starcoin_proof_structured_hash::hash(SPARSE_MERKLE_INTERNAL_NODE, &internal_node);
            i = i + 1;
        };
        current_hash
    }

    #[test]
    public fun test_starcoin_proof_verify_is_expect_root() {
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
        let actual_root_hash = Self::computer_root_hash(
            element_key,
            x"4f2b59b9af93b435e0a33b6ab7a8a90e471dba936be2bc2937629b7782b8ebd0",
            siblings,
        );
        assert!(actual_root_hash == expect_root_hash, 1000);
    }
}


