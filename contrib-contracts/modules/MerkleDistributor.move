address StarcoinAssociation {
module MerkleDistributorScripts {
    use StarcoinAssociation::MerkleDistributor;
    use StarcoinFramework::Account;
    public(script) fun create<T: store>(signer: signer, merkle_root: vector<u8>, token_amounts: u128, leafs: u64) {
        let tokens = Account::withdraw<T>(&signer, token_amounts);
        MerkleDistributor::create<T>(&signer, merkle_root, tokens, leafs);
    }

    public(script) fun claim_for_address<T: store>(distribution_address: address, index: u64, account: address, amount: u128, merkle_proof: vector<vector<u8>>) {
        MerkleDistributor::claim_for_address<T>(distribution_address, index, account, amount, merkle_proof);
    }
    public(script) fun claim<T: store>(signer: signer, distribution_address: address, index: u64, amount: u128, merkle_proof: vector<vector<u8>>) {
        let tokens = MerkleDistributor::claim<T>(&signer, distribution_address, index, amount, merkle_proof);
        Account::deposit_to_self<T>(&signer, tokens);
    }
}

module MerkleProof {
    use StarcoinFramework::Hash;
    use StarcoinFramework::Vector;
    use StarcoinFramework::Compare;

    /// verify leaf node with hash of `leaf` with `proof` againest merkle `root`.
    public fun verify(proof: &vector<vector<u8>>, root: &vector<u8>, leaf: vector<u8>): bool {
        let computed_hash = leaf;
        let i = 0;
        let proof_length = Vector::length(proof);
        while(i < proof_length) {
            let sibling = Vector::borrow(proof, i);
            // computed_hash is left.
            if (Compare::cmp_bytes(&computed_hash,sibling) < 2) {
                let concated = concat(computed_hash, *sibling);
                computed_hash = Hash::sha3_256(concated);
            } else {
                let concated = concat(*sibling, computed_hash);
                computed_hash = Hash::sha3_256(concated);

            };

            i = i+1;
        };
        &computed_hash == root
    }


    fun concat(v1: vector<u8>, v2: vector<u8>): vector<u8> {
        Vector::append(&mut v1, v2);
        v1
    }
}



module MerkleDistributor {
    use StarcoinFramework::Token::{Token, Self};
    use StarcoinAssociation::MerkleProof;
    use StarcoinFramework::Vector;
    use StarcoinFramework::BCS;
    // use StarcoinFramework::BitOperators;
    use StarcoinFramework::Account;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Hash;

    struct MerkleDistribution<T: store> has key {
        merkle_root: vector<u8>,
        tokens: Token<T>,
        claimed_bitmap: vector<u128>,
    }
    const INVALID_PROOF: u64 = 1;
    const ALREADY_CLAIMED: u64 = 2;

    /// Initialization.
    public fun create<T: store>(signer: &signer, merkle_root: vector<u8>, tokens: Token<T>, leafs: u64) {
        let bitmap_count = leafs / 128;
        if (bitmap_count * 128 < leafs) {
            bitmap_count = bitmap_count + 1;
        };
        let claimed_bitmap = Vector::empty();
        let j = 0;
        while (j < bitmap_count) {
            Vector::push_back(&mut claimed_bitmap, 0u128);
            j = j + 1;
        };
        let distribution = MerkleDistribution{
            merkle_root,
            tokens,
            claimed_bitmap
        };
        move_to(signer, distribution);
    }

    /// claim for some address.
    public fun claim_for_address<T: store>(distribution_address: address, index: u64, account: address, amount: u128, merkle_proof: vector<vector<u8>>)
    acquires  MerkleDistribution {
        let distribution = borrow_global_mut<MerkleDistribution<T>>(distribution_address);
        let claimed_tokens = internal_claim(distribution, index, account, amount, merkle_proof);
        Account::deposit(account, claimed_tokens);
    }

    /// claim by myself.
    public fun claim<T: store>(signer: &signer, distribution_address: address, index: u64, amount: u128, merkle_proof: vector<vector<u8>>): Token<T>
    acquires  MerkleDistribution  {
        let distribution = borrow_global_mut<MerkleDistribution<T>>(distribution_address);
        internal_claim(distribution, index, Signer::address_of(signer), amount, merkle_proof)
    }

    /// Query whether `index` of `distribution_address` has already claimed.
    public fun is_claimed<T: store>(distribution_address: address, index: u64): bool
    acquires MerkleDistribution {
        let distribution = borrow_global<MerkleDistribution<T>>(distribution_address);
        is_claimed_(distribution, index)
    }

    fun internal_claim<T: store>(distribution: &mut MerkleDistribution<T>, index: u64, account: address, amount: u128, merkle_proof: vector<vector<u8>>): Token<T> {
        let claimed =  is_claimed_(distribution, index);
        assert(!claimed, Errors::custom(ALREADY_CLAIMED));

        let leaf_data = encode_leaf(&index, &account, &amount);
        let verified = MerkleProof::verify(&merkle_proof, &distribution.merkle_root, Hash::sha3_256(leaf_data));
        assert(verified, Errors::custom(INVALID_PROOF));

        set_claimed_(distribution, index);

        Token::withdraw(&mut distribution.tokens, amount)
    }

    fun is_claimed_<T: store>(distribution: &MerkleDistribution<T>, index: u64): bool {
        let claimed_word_index = index / 128;
        let claimed_bit_index = ((index % 128) as u8);
        let word = Vector::borrow(&distribution.claimed_bitmap, claimed_word_index);
        let mask = 1u128 << claimed_bit_index;
        (*word & mask) == mask
    }

    fun set_claimed_<T: store>(distribution: &mut MerkleDistribution<T>, index: u64) {
        let claimed_word_index = index / 128;
        let claimed_bit_index = ((index % 128) as u8);
        let word = Vector::borrow_mut(&mut distribution.claimed_bitmap, claimed_word_index);
        // word | (1 << bit_index)
        let mask = 1u128 << claimed_bit_index;
        *word = (*word | mask);
    }

    fun encode_leaf(index: &u64, account: &address, amount: &u128): vector<u8> {
        let leaf = Vector::empty();
        Vector::append(&mut leaf, BCS::to_bytes(index));
        Vector::append(&mut leaf, BCS::to_bytes(account));
        Vector::append(&mut leaf, BCS::to_bytes(amount));
        leaf
    }
}
}