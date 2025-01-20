address StarcoinAssociation {
module MerkleDistributorScripts {
    use StarcoinAssociation::MerkleDistributor;
    use starcoin_framework::coin;
    use starcoin_std::signer;
    public entry fun create<T>(signer: signer, merkle_root: vector<u8>, token_amounts: u128, leaves: u64) {
        let coins = coin::withdraw<T>(&signer, (token_amounts as u64));
        MerkleDistributor::create<T>(&signer, merkle_root, coins, leaves);
    }

    public entry fun claim_for_address<T>(singer: signer, distribution_address: address, index: u64, account: address, amount: u128, merkle_proof: vector<vector<u8>>) {
        MerkleDistributor::claim_for_address<T>(&singer,distribution_address, index, account, amount, merkle_proof);
    }
    public entry fun claim<T>(signer: signer, distribution_address: address, index: u64, amount: u128, merkle_proof: vector<vector<u8>>) {
        let coins = MerkleDistributor::claim<T>(&signer, distribution_address, index, amount, merkle_proof);
        let account_addr = signer::address_of(&signer);
        coin::deposit<T>(account_addr, coins);
    }
}

module MerkleProof {
    use std::hash;
    use std::vector;
    use starcoin_std::comparator;

    /// verify leaf node with hash of `leaf` with `proof` against merkle `root`.
    public fun verify(proof: &vector<vector<u8>>, root: &vector<u8>, leaf: vector<u8>): bool {
        let computed_hash = leaf;
        let i = 0;
        let proof_length = vector::length(proof);
        while(i < proof_length) {
            let sibling = vector::borrow(proof, i);
            // computed_hash is left.
            if (!comparator::is_greater_than(&comparator::compare_u8_vector(computed_hash, *sibling))) {
                let concated = concat(computed_hash, *sibling);
                computed_hash = hash::sha3_256(concated);
            } else {
                let concated = concat(*sibling, computed_hash);
                computed_hash = hash::sha3_256(concated);

            };

            i = i+1;
        };
        &computed_hash == root
    }


    fun concat(v1: vector<u8>, v2: vector<u8>): vector<u8> {
        vector::append(&mut v1, v2);
        v1
    }
}



module MerkleDistributor {
    use StarcoinAssociation::MerkleProof;
    use starcoin_framework::coin;
    use std::bcs;
    use std::error;
    use std::hash;
    use std::vector;
    use starcoin_std::signer;

    struct MerkleDistribution<phantom T> has key {
        merkle_root: vector<u8>,
        coins: coin::Coin<T>,
        claimed_bitmap: vector<u128>,
    }
    const INVALID_PROOF: u64 = 1;
    const ALREADY_CLAIMED: u64 = 2;

    /// Initialization.
    public fun create<T>(signer: &signer, merkle_root: vector<u8>, coins: coin::Coin<T>, leaves: u64) {
        let bitmap_count = leaves / 128;
        if (bitmap_count * 128 < leaves) {
            bitmap_count = bitmap_count + 1;
        };
        let claimed_bitmap = vector::empty();
        let j = 0;
        while (j < bitmap_count) {
            vector::push_back(&mut claimed_bitmap, 0u128);
            j = j + 1;
        };
        let distribution = MerkleDistribution{
            merkle_root,
            coins,
            claimed_bitmap
        };
        move_to(signer, distribution);
    }

    /// claim for some address.
    public fun claim_for_address<T>(signer: &signer, distribution_address: address, index: u64, account: address, amount: u128, merkle_proof: vector<vector<u8>>)
    acquires  MerkleDistribution {
        let distribution = borrow_global_mut<MerkleDistribution<T>>(distribution_address);
        let claimed_tokens = internal_claim(signer, distribution, index, account, amount, merkle_proof);
        coin::deposit(account, claimed_tokens);
    }

    /// claim by myself.
    public fun claim<T>(signer: &signer, distribution_address: address, index: u64, amount: u128, merkle_proof: vector<vector<u8>>): coin::Coin<T>
    acquires  MerkleDistribution  {
        let distribution = borrow_global_mut<MerkleDistribution<T>>(distribution_address);
        internal_claim(signer, distribution, index, signer::address_of(signer), amount, merkle_proof)
    }

    /// Query whether `index` of `distribution_address` has already claimed.
    public fun is_claimed<T>(distribution_address: address, index: u64): bool
    acquires MerkleDistribution {
        let distribution = borrow_global<MerkleDistribution<T>>(distribution_address);
        is_claimed_(distribution, index)
    }

    fun internal_claim<T>(signer: &signer, distribution: &mut MerkleDistribution<T>, index: u64, account: address, amount: u128, merkle_proof: vector<vector<u8>>): coin::Coin<T> {
        let claimed =  is_claimed_(distribution, index);
      //  assert!(!claimed, error::invalid_argument(ALREADY_CLAIMED));

        let leaf_data = encode_leaf(&index, &account, &amount);
        let verified = MerkleProof::verify(&merkle_proof, &distribution.merkle_root, hash::sha3_256(leaf_data));
        assert!(verified, error::invalid_argument(INVALID_PROOF));

        set_claimed_(distribution, index);

        coin::withdraw(signer, (amount as u64))
    }

    fun is_claimed_<T>(distribution: &MerkleDistribution<T>, index: u64): bool {
        let claimed_word_index = index / 128;
        let claimed_bit_index = ((index % 128) as u8);
        let word = vector::borrow(&distribution.claimed_bitmap, claimed_word_index);
        let mask = 1u128 << claimed_bit_index;
        (*word & mask) == mask
    }

    fun set_claimed_<T>(distribution: &mut MerkleDistribution<T>, index: u64) {
        let claimed_word_index = index / 128;
        let claimed_bit_index = ((index % 128) as u8);
        let word = vector::borrow_mut(&mut distribution.claimed_bitmap, claimed_word_index);
        // word | (1 << bit_index)
        let mask = 1u128 << claimed_bit_index;
        *word = (*word | mask);
    }

    fun encode_leaf(index: &u64, account: &address, amount: &u128): vector<u8> {
        let leaf = vector::empty();
        vector::append(&mut leaf, bcs::to_bytes(index));
        vector::append(&mut leaf, bcs::to_bytes(account));
        vector::append(&mut leaf, bcs::to_bytes(amount));
        leaf
    }
}
}