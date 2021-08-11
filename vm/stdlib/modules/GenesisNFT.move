module 0x1::GenesisNFT {
    use 0x1::IdentifierNFT;
    use 0x1::Option::Option;
    use 0x1::NFT::{Self, MintCapability};
    use 0x1::MerkleNFTDistributor;
    use 0x1::CoreAddresses;

    struct GenesisNFT has store{}
    struct GenesisNFTMeta has copy, store, drop{
        index: u64
    }
    struct GenesisNFTInfo has copy, store, drop{
        merkle_root: vector<u8>,
        total_supply: u64,
    }
    struct GenesisNFTMintCapability has key{
        cap: MintCapability<GenesisNFTMeta>
    }
    public fun initialize(sender: &signer, merkle_root: vector<u8>, leafs: u64, image: vector<u8>){
        CoreAddresses::assert_genesis_address(sender);
        let metadata = NFT::new_meta_with_image(b"StarcoinGenesisNFT", image, b"The starcoin genesis NFT");
        let  nft_info = GenesisNFTInfo{merkle_root: *&merkle_root, total_supply: leafs};
        let cap = MerkleNFTDistributor::register<GenesisNFTMeta, GenesisNFTInfo>(sender, merkle_root, leafs, nft_info, metadata);
        move_to(sender, GenesisNFTMintCapability{cap});
    }

    public fun mint(sender: &signer, index: u64, merkle_proof:vector<vector<u8>>)
        acquires GenesisNFTMintCapability{
            let metadata = NFT::empty_meta();
            let cap = borrow_global_mut<GenesisNFTMintCapability>(CoreAddresses::GENESIS_ADDRESS());
            let nft = MerkleNFTDistributor::mint_with_cap<GenesisNFTMeta, GenesisNFT, GenesisNFTInfo>(sender, &mut cap.cap, CoreAddresses::GENESIS_ADDRESS(), index, metadata, GenesisNFTMeta{index}, GenesisNFT{}, merkle_proof);
            IdentifierNFT::grant(&mut cap.cap, sender, nft);
        }

    public fun verify(account: address, index: u64, merkle_proof: vector<vector<u8>>): bool {
             MerkleNFTDistributor::verify_proof<GenesisNFTMeta>(account, CoreAddresses::GENESIS_ADDRESS(), index, merkle_proof)
        }

    public fun get_info(owner: address): Option<NFT::NFTInfo<GenesisNFTMeta>>{
        IdentifierNFT::get_nft_info<GenesisNFTMeta, GenesisNFT>(owner)
    }

    public fun is_minted(index: u64): bool {
        let creator = CoreAddresses::GENESIS_ADDRESS();
        MerkleNFTDistributor::is_minted<GenesisNFTMeta>(creator, index)
    }
}

module 0x1::GenesisNFTScripts {
    use 0x1::GenesisNFT;

    /// Mint a GenesisNFT
    public(script) fun mint(sender: signer, index: u64, merkle_proof:vector<vector<u8>>) {
        GenesisNFT::mint(&sender, index, merkle_proof);
    }
}
