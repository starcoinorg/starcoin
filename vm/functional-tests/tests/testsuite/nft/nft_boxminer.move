//! account: creator
//! account: alice, 0x07fa08a855753f0ff7292fdcbe871216, 100 0x1::STC::STC

//! sender: creator
address creator = {{creator}};
module creator::BoxMiner {
    use 0x1::NFT::{Self, NFT, MintCapability};
    use 0x1::Account;
    use 0x1::NFTGallery;
    use 0x1::STC::STC;

    struct BoxMiner has copy, store, drop{
        price: u128,
    }

    struct NFTInfo has copy, store, drop{
        total_supply: u64,
        price: u128,
    }
    struct BoxMinerBody has store{}

    struct BoxMinerMintCapability has key{
        cap: MintCapability<BoxMiner>,
    }

    public fun init(sender: &signer, total_supply:u64, price: u128){
        let meta = NFT::new_meta_with_image(b"stc_box_miner_nft", b"ipfs:://xxx", b"This is the starcoin boxminer nft");
        let nft_type_info = NFT::new_nft_type_info(sender, NFTInfo{total_supply, price}, meta);
        NFT::register<BoxMiner, NFTInfo>(sender, nft_type_info);
        let cap = NFT::remove_mint_capability<BoxMiner>(sender);
        move_to(sender, BoxMinerMintCapability{cap});
    }

    public fun do_accept(sender: &signer) {
        NFTGallery::accept<BoxMiner, BoxMinerBody>(sender);
    }

    public fun mint(sender: &signer): NFT<BoxMiner, BoxMinerBody> acquires BoxMinerMintCapability{
        let ex_info = NFT::nft_type_info_ex_info<BoxMiner, NFTInfo>();
        let counter = NFT::nft_type_info_counter<BoxMiner, NFTInfo>();
        let total_supply = ex_info.total_supply;
        let price = ex_info.price;
        assert(total_supply >= counter, 1000);
        let tokens = Account::withdraw<STC>(sender, price);
        Account::deposit<STC>(@creator, tokens);
        let cap = borrow_global_mut<BoxMinerMintCapability>(@creator);
        let metadata = NFT::new_meta(b"stc_box_miner", b"This is the starcoin boxminer.");
        let nft = NFT::mint_with_cap<BoxMiner, BoxMinerBody, NFTInfo>(@creator, &mut cap.cap, metadata, BoxMiner{price}, BoxMinerBody{});
        return nft
    }
}

// check: EXECUTED

//! new-transaction
//! sender: creator
address creator = {{creator}};
script {
    use creator::BoxMiner;
    fun main(sender: signer) {
        BoxMiner::init(&sender, 2u64, 100u128);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
address creator = {{creator}};
script {
    use creator::BoxMiner;
    use 0x1::NFTGallery;
    fun main(sender: signer) {
        let nft = BoxMiner::mint(&sender);
        BoxMiner::do_accept(&sender);
        NFTGallery::deposit(&sender, nft);
}
}

// check: EXECUTED