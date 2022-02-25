//# init -n dev

//# faucet --addr creator --amount 100000000000

//# faucet --addr alice

//# publish
module creator::BoxMiner {
    use StarcoinFramework::NFT::{Self, NFT, MintCapability};
    use StarcoinFramework::Account;
    use StarcoinFramework::NFTGallery;
    use StarcoinFramework::STC::STC;
    use StarcoinFramework::Signer;

    struct BoxMiner has copy, store, drop{
        price: u128,
    }

    struct NFTInfo has copy, store, drop, key{
        total_supply: u64,
        price: u128,
    }
    struct BoxMinerBody has store{}

    struct BoxMinerMintCapability has key{
        cap: MintCapability<BoxMiner>,
    }

    public fun init(sender: &signer, total_supply:u64, price: u128){
        assert!(Signer::address_of(sender) == @creator, 1000);
        let meta = NFT::new_meta_with_image(b"stc_box_miner_nft", b"ipfs:://xxx", b"This is the starcoin boxminer nft");
        NFT::register_v2<BoxMiner>(sender, meta);
        move_to(sender, NFTInfo{total_supply, price});
        let cap = NFT::remove_mint_capability<BoxMiner>(sender);
        move_to(sender, BoxMinerMintCapability{cap});
    }

    public fun do_accept(sender: &signer) {
        NFTGallery::accept<BoxMiner, BoxMinerBody>(sender);
    }

    public fun mint(sender: &signer): NFT<BoxMiner, BoxMinerBody> acquires BoxMinerMintCapability, NFTInfo{
        let ex_info = borrow_global<NFTInfo>(@creator);
        let counter = NFT::nft_type_info_counter_v2<BoxMiner>();
        let total_supply = ex_info.total_supply;
        let price = ex_info.price;
        assert!(total_supply >= counter, 1000);
        let tokens = Account::withdraw<STC>(sender, price);
        Account::deposit<STC>(@creator, tokens);
        let cap = borrow_global_mut<BoxMinerMintCapability>(@creator);
        let metadata = NFT::new_meta(b"stc_box_miner", b"This is the starcoin boxminer.");
        let nft = NFT::mint_with_cap_v2<BoxMiner, BoxMinerBody>(@creator, &mut cap.cap, metadata, BoxMiner{price}, BoxMinerBody{});
        return nft
    }
}

// check: EXECUTED

//# run --signers creator
script {
    use creator::BoxMiner;
    fun main(sender: signer) {
        BoxMiner::init(&sender, 2u64, 100u128);
    }
}

// check: EXECUTED

//# run --signers alice
script {
    use creator::BoxMiner;
    use StarcoinFramework::NFTGallery;
    fun main(sender: signer) {
        let nft = BoxMiner::mint(&sender);
        NFTGallery::deposit(&sender, nft);
}
}

// check: EXECUTED