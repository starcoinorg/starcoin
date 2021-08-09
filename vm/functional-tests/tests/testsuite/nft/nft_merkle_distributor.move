//! account: creator
//! account: genesis
//! account: alice
//! sender: genesis
address genesis= {{genesis}};
module genesis::MerkleProof {
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
//! sender: genesis
address genesis= {{genesis}};
module genesis::MerkleNFTDistributor {
    use 0x1::Vector;
    use 0x1::NFT::{Self, NFT, Metadata, MintCapability,NFTTypeInfo};
    use 0x1::Hash;
    use 0x1::BCS;
    use 0x1::Signer;
    use 0x1::Errors;
    use genesis::MerkleProof;
    const ALREADY_MINTED: u64 = 1000;
    const INVALID_PROOF: u64 = 1001;
    const ERR_NO_MINT_CAPABILITY: u64 = 1002;

    struct MerkleNFTDistribution<NFTMeta: copy + store + drop> has key {
        merkle_root: vector<u8>,
        claimed_bitmap: vector<u128>,
    }

    struct MerkleNFTDistributorMintCapability<NFTMeta: store> has key {
        cap: MintCapability<NFTMeta>,
    }

    public fun register<NFTMeta: copy + store + drop, Info: copy + store + drop>(signer: &signer, merkle_root: vector<u8>, leafs: u64, type_info: NFTTypeInfo<NFTMeta, Info>): MintCapability<NFTMeta> {
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
        NFT::register<NFTMeta, Info>(signer, type_info);
        move_to(signer, distribution);
        NFT::remove_mint_capability<NFTMeta>(signer)
    }

    public fun mint_with_cap<NFTMeta: copy + store + drop, NFTBody: store, Info: copy + store + drop>(sender: &signer, cap:&mut MintCapability<NFTMeta>, creator:address, index: u64, base_meta: Metadata, type_meta: NFTMeta, body: NFTBody, merkle_proof:vector<vector<u8>>): NFT<NFTMeta, NFTBody>
        acquires MerkleNFTDistribution {
            let addr = Signer::address_of(sender);
            let distribution = borrow_global_mut<MerkleNFTDistribution<NFTMeta>>(creator);
            let minted = is_minted_<NFTMeta>(distribution, index);
            assert(!minted, Errors::custom(ALREADY_MINTED));
            let leaf_data = encode_leaf(&index, &addr);
            let verified = MerkleProof::verify(&merkle_proof, &distribution.merkle_root, Hash::sha3_256(leaf_data));
            assert(verified, Errors::custom(INVALID_PROOF));
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

    fun is_minted_<NFTMeta: copy + store + drop>(distribution: &MerkleNFTDistribution<NFTMeta>, index: u64): bool {
        let claimed_word_index = index / 128;
        let claimed_bit_index = ((index % 128) as u8);
        let word = Vector::borrow( &distribution.claimed_bitmap, claimed_word_index);
        let mask = 1u128 << claimed_bit_index;
        (*word & mask) == mask
    }

}

// check: EXECUTED


//! new-transaction
//! sender: creator
address creator = {{creator}};
module creator::GenesisNFT {
    use 0x1::IdentifierNFT;
    use 0x1::Signer;
    use 0x1::Option::Option;
    use 0x1::NFT::{Self, MintCapability};
    use genesis::MerkleNFTDistributor;
    struct GenesisNFT has store{}
    //TODO: write block height, hash, timestamp or something to it.
    struct GenesisNFTMeta has copy, store, drop{}
    struct GenesisNFTInfo has copy, store, drop{}
    struct GenesisNFTMintCapability has key{
        cap: MintCapability<GenesisNFTMeta>
    }
    public fun init(sender: &signer, merkle_root: vector<u8>, leafs: u64){
        assert(Signer::address_of(sender) == @creator, 1000);
        let metadata = NFT::new_meta_with_image(b"StarcoinGenesisNFT", b"ipfs:://xxxxxx", b"The starcoin genesis NFT");
        let nft_type_info=NFT::new_nft_type_info(sender, GenesisNFTInfo{}, metadata);
        let cap = MerkleNFTDistributor::register<GenesisNFTMeta, GenesisNFTInfo>(sender, merkle_root, leafs, nft_type_info);
        move_to(sender, GenesisNFTMintCapability{cap});
    }

    public fun mint(sender: &signer, index: u64, merkle_proof:vector<vector<u8>>)
        acquires GenesisNFTMintCapability{
            let metadata = NFT::new_meta(b"StarcoinGenesisNFT", b"The starcoin genesis NFT");
            let cap = borrow_global_mut<GenesisNFTMintCapability>(@creator);
            let nft = MerkleNFTDistributor::mint_with_cap<GenesisNFTMeta, GenesisNFT, GenesisNFTInfo>(sender, &mut cap.cap, @creator, index, metadata, GenesisNFTMeta{}, GenesisNFT{}, merkle_proof);
            IdentifierNFT::grant(&mut cap.cap, sender, nft);
        }
    public fun get_info(owner: address): Option<NFT::NFTInfo<GenesisNFTMeta>>{
        IdentifierNFT::get_nft_info<GenesisNFTMeta, GenesisNFT>(owner)

    }
}


// check: EXECUTED


//! new-transaction
//! sender: creator
address creator={{creator}};
script {
    use creator::GenesisNFT;
    fun main(sender: signer) {
        let root = b"";
        GenesisNFT::init(&sender, root, 2);
    }
}


// check: EXECUTED

//! new-transaction
//! sender: alice
address creator={{creator}};
script {
    use 0x1::Vector;
    use 0x1::Signer;
    use 0x1::Option;
    use 0x1::NFT;
    use creator::GenesisNFT::{Self, GenesisNFTMeta};
    //TODO: generate the real root and proof
    fun main(sender: signer) {
        let proof = Vector::empty<vector<u8>>();
        GenesisNFT::mint(&sender, 1, proof);
        let info = GenesisNFT::get_info(Signer::address_of(&sender));
        assert(Option::is_some<NFT::NFTInfo<GenesisNFTMeta>>(&info), 1000);
    }
}

// check:  VMExecutionFailure(ABORTED { code: 256511
