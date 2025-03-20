module StarcoinFramework::MerkleProof {
    use StarcoinFramework::Hash;
    use StarcoinFramework::Vector;
    use StarcoinFramework::Compare;

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

module StarcoinFramework::MerkleNFTDistributor {
    use StarcoinFramework::Vector;
    use StarcoinFramework::NFT::{Self, NFT, Metadata, MintCapability};
    use StarcoinFramework::Hash;
    use StarcoinFramework::BCS;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Errors;
    use StarcoinFramework::MerkleProof;
    const ALREADY_MINTED: u64 = 1000;
    const INVALID_PROOF: u64 = 1001;
    const ERR_NO_MINT_CAPABILITY: u64 = 1002;

    struct MerkleNFTDistribution<phantom NFTMeta: copy + store + drop> has key {
        merkle_root: vector<u8>,
        claimed_bitmap: vector<u128>,
    }

    public fun register<NFTMeta: copy + store + drop, Info: copy + store + drop>(signer: &signer, merkle_root: vector<u8>, leafs: u64, info: Info, meta: Metadata): MintCapability<NFTMeta> {
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
        NFT::register<NFTMeta, Info>(signer, info, meta);
        move_to(signer, distribution);
        NFT::remove_mint_capability<NFTMeta>(signer)
    }

    public fun mint_with_cap<NFTMeta: copy + store + drop, NFTBody: store, Info: copy + store + drop>(sender: &signer, cap:&mut MintCapability<NFTMeta>, creator: address, index: u64, base_meta: Metadata, type_meta: NFTMeta, body: NFTBody, merkle_proof:vector<vector<u8>>): NFT<NFTMeta, NFTBody>
        acquires MerkleNFTDistribution {
            let addr = Signer::address_of(sender);
            let distribution = borrow_global_mut<MerkleNFTDistribution<NFTMeta>>(creator);
            let minted = is_minted_<NFTMeta>(distribution, index);
            assert!(!minted, Errors::custom(ALREADY_MINTED));
            let leaf_data = encode_leaf(&index, &addr);
            let verified = MerkleProof::verify(&merkle_proof, &distribution.merkle_root, Hash::sha3_256(leaf_data));
            assert!(verified, Errors::custom(INVALID_PROOF));
            set_minted_(distribution, index);
            let nft = NFT::mint_with_cap<NFTMeta, NFTBody, Info>(creator, cap, base_meta, type_meta, body);
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

    public fun verify_proof<NFTMeta: copy + store + drop>(account: address, creator: address, index: u64, merkle_proof:vector<vector<u8>>): bool
        acquires MerkleNFTDistribution {
            let distribution = borrow_global_mut<MerkleNFTDistribution<NFTMeta>>(creator);
            let leaf_data = encode_leaf(&index, &account);
            MerkleProof::verify(&merkle_proof, &distribution.merkle_root, Hash::sha3_256(leaf_data))
        }

    public fun is_minted<NFTMeta: copy + store + drop>(creator: address, index: u64): bool
        acquires MerkleNFTDistribution {
            let distribution = borrow_global_mut<MerkleNFTDistribution<NFTMeta>>(creator);
            is_minted_<NFTMeta>(distribution, index)
        }

    fun is_minted_<NFTMeta: copy + store + drop>(distribution: &MerkleNFTDistribution<NFTMeta>, index: u64): bool {
        let claimed_word_index = index / 128;
        let claimed_bit_index = ((index % 128) as u8);
        let word = Vector::borrow( &distribution.claimed_bitmap, claimed_word_index);
        let mask = 1u128 << claimed_bit_index;
        (*word & mask) == mask
    }

}
