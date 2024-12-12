address StarcoinAssociation {
    module StarcoinVerifierScripts {
        use StarcoinAssociation::StarcoinVerifier;
        public entry fun create(signer: signer, merkle_root: vector<u8>) {
            StarcoinVerifier::create(&signer, merkle_root);
        }
    }
    module StarcoinVerifier {
        use std::vector;
        use StarcoinAssociation::Bit;
        use StarcoinAssociation::StructuredHash;
        use std::hash;

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

        public fun verify_on(merkle_address: address, account_address: vector<u8>, account_state_root_hash: vector<u8>, proofs: vector<vector<u8>>): bool
        acquires StarcoinMerkle  {
            let merkle = borrow_global<StarcoinMerkle>(merkle_address);

            verify(*&merkle.merkle_root, account_address, account_state_root_hash, proofs)
        }

        public fun verify(expected_root: vector<u8>, account_address: vector<u8>, account_state_root_hash: vector<u8>, proofs: vector<vector<u8>>): bool {
            let address_hash = hash::sha3_256(account_address);
            let leaf_node = Node { hash1: copy address_hash, hash2: account_state_root_hash};
            let current_hash = StructuredHash::hash(SPARSE_MERKLE_LEAF_NODE, &leaf_node);
            let i = 0;
            let proof_length = vector::length(&proofs);
            while (i < proof_length) {
                let sibling = *vector::borrow(&proofs, i);
                let bit = Bit::get_bit(&address_hash, proof_length - i - 1);
                let internal_node = if (bit) {
                    Node {hash1: sibling, hash2: current_hash}
                } else {
                    Node {hash1: current_hash, hash2: sibling}
                };
                current_hash = StructuredHash::hash(SPARSE_MERKLE_INTERNAL_NODE, &internal_node);
                i = i+1;
            };
            current_hash == expected_root
        }
    }

    module StructuredHash {
        use std::hash;
        use std::vector;
        use std::bcs;
        const STARCOIN_HASH_PREFIX: vector<u8> = b"STARCOIN::";
        public fun hash<MoveValue: store>(structure: vector<u8>, data: &MoveValue): vector<u8> {
            let prefix_hash = hash::sha3_256(concat(&STARCOIN_HASH_PREFIX, structure));
            let bcs_bytes = bcs::to_bytes(data);
            Hash::sha3_256(concat(&prefix_hash, bcs_bytes))
        }

        fun concat(v1: &vector<u8>, v2: vector<u8>): vector<u8> {
            let data = *v1;
            vector::append(&mut data, v2);
            data
        }

    }
    module Bit {
        use std::vector;
        public fun get_bit(data: &vector<u8>, index: u64): bool {
            let pos = index / 8;
            let bit = (7 - index % 8);
            (*vector::borrow(data, pos) >> (bit as u8)) & 1u8 != 0
        }
    }
}