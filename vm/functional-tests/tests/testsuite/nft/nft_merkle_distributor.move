//! account: creator

//! sender: creator
address creator = {{creator}};
module creator::MerkleProof {
    use 0x1::Hash;
    use 0x1::Vector;
    use 0x1::Compare;

    /// verify leaf node with hash of `leaf` with `proof` againest merkle `root`.
    public fun verify(proof: &vector<vector<u8>>, root: &vector<u8>, leaf: vector<u8>): bool {
        let computed_hash = leaf;
        let i = 0;
        let proof_length = Vector::length(proof);
        while (i < proof_length) {
            let sibling = Vector::borrow(proof, i);
            // computed_hash is left.
            if (Compare::cmp_bytes( &computed_hash, sibling) < 2) {
                let concated = concat(computed_hash, * sibling);
                computed_hash = Hash::sha3_256(concated);
            } else {
                let concated = concat(*sibling, computed_hash);
                computed_hash = Hash::sha3_256(concated);
            };

            i = i + 1;
        };
        &computed_hash == root
    }

    fun concat(v1: vector<u8>, v2: vector<u8>): vector<u8> {
        Vector::append( &mut v1, v2);
        v1
    }
}

// check: EXECUTED


//! new-transaction
//! sender: creator
address creator = {{creator}};
module creator::MerkleNFTDistributor {
    use 0x1::Vector;
    use 0x1::NFT::{Self, NFT, Metadata};
    use 0x1::Hash;
    use 0x1::BCS;
    use 0x1::Signer;
    use 0x1::Errors;
    use creator::MerkleProof;
    const ALREADY_MINTED: u64 = 1000;
    const INVALID_PROOF:u64 = 1001;
    struct MerkleNFTDistribution<NFTMeta: copy + store + drop> has key {
        merkle_root: vector<u8>,
        claimed_bitmap: vector<u128>,
    }

    public fun init<NFTMeta: copy + store + drop, Info: copy + store + drop>(signer: &signer, merkle_root: vector<u8>, leafs: u64, info: Info) {
        let bitmap_count = leafs / 128;
        if (bitmap_count * 128 < leafs) {
            bitmap_count = bitmap_count + 1;
        };
        let claimed_bitmap = Vector::empty();
        let j = 0;
        while (j < bitmap_count) {
            Vector::push_back( &mut claimed_bitmap, 0u128);
            j = j + 1;
        };
        let distribution = MerkleNFTDistribution<NFTMeta>{
            merkle_root,
            claimed_bitmap
        };
        NFT::register<NFTMeta, Info>(signer, info);
        move_to(signer, distribution);
    }

    fun mint<NFTMeta: copy + store + drop, NFTBody: store, Info: copy + store + drop>(sender: &signer, index: u64, base_meta: Metadata, type_meta: NFTMeta, body: NFTBody, merkle_proof:vector<vector<u8>>): NFT<NFTMeta, NFTBody>
        acquires MerkleNFTDistribution {
            let distribution = borrow_global_mut<MerkleNFTDistribution<NFTMeta>>(@creator);
            let addr = Signer::address_of(sender);
            let minted = is_minted_<NFTMeta>(distribution, index);
            assert(!minted, Errors::custom(ALREADY_MINTED));
            let leaf_data = encode_leaf(&index, &addr);
            let verified = MerkleProof::verify(&merkle_proof, &distribution.merkle_root, Hash::sha3_256(leaf_data));
            assert(verified, Errors::custom(INVALID_PROOF));
            set_minted_(distribution, index);
            let nft = NFT::mint<NFTMeta, NFTBody, Info>(sender, base_meta, type_meta, body);
            return nft
        }

    fun encode_leaf(index: &u64, account: &address): vector<u8> {
        let leaf = Vector::empty();
        Vector::append(&mut leaf, BCS::to_bytes(index));
        Vector::append(&mut leaf, BCS::to_bytes(account));
        leaf
    }

    fun set_minted_<NFTMeta: copy + store + drop>(distribution: &mut MerkleNFTDistribution<NFTMeta>, index: u64) {
        let claimed_word_index = index / 128;
        let claimed_bit_index = ((index % 128) as u8);
        let word = Vector::borrow_mut(&mut distribution.claimed_bitmap, claimed_word_index);
        // word | (1 << bit_index)
        let mask = 1u128 << claimed_bit_index;
        *word = (*word | mask);
    }

    fun is_minted_<NFTMeta: copy + store + drop>(distribution: &MerkleNFTDistribution<NFTMeta>, index: u64): bool {
        let claimed_word_index = index / 128;
        let claimed_bit_index = ((index % 128) as u8);
        let word = Vector::borrow( &distribution.claimed_bitmap, claimed_word_index);
        let mask = 1u128 << claimed_bit_index;
        (*word & mask) == mask
    }

}

// check: EXECUTED
