//# init -n dev

//# faucet --addr creator

//# faucet --addr bob


//# publish
module creator::AnyNFT {
    use StarcoinFramework::NFT::{Self, NFT, MintCapability, BurnCapability};
    use StarcoinFramework::NFTGallery;
    use StarcoinFramework::Signer;
    struct AnyNFT has copy, store, drop{}
    struct AnyNFTBody has store{
    }

    struct AnyNFTMintCapability has key{
        cap: MintCapability<AnyNFT>,
    }

    struct AnyNFTBurnCapability has key{
        cap: BurnCapability<AnyNFT>,
    }

    public fun init(sender: &signer){
        NFT::register_v2<AnyNFT>(sender,NFT::empty_meta());
        let cap = NFT::remove_mint_capability<AnyNFT>(sender);
        move_to(sender, AnyNFTMintCapability{cap});
        let cap = NFT::remove_burn_capability<AnyNFT>(sender);
        move_to(sender, AnyNFTBurnCapability{cap});
        Self::do_accept(sender);
    }

    public fun mint(sender: &signer) acquires AnyNFTMintCapability{
        let sender_addr = Signer::address_of(sender);
        let cap = borrow_global_mut<AnyNFTMintCapability>(@creator);
        let metadata = NFT::new_meta_with_image(b"test_nft_1", b"ipfs:://xxxxxx", b"This is a test nft.");
        let nft = NFT::mint_with_cap_v2<AnyNFT,AnyNFTBody>(sender_addr, &mut cap.cap, metadata, AnyNFT{}, AnyNFTBody{});
        NFTGallery::deposit(sender, nft);
    }

    public fun mint_many(sender: &signer, amount: u64) acquires AnyNFTMintCapability{
        let i = 0;
        while (i < amount ) {
            mint(sender);
            i = i + 1;
        }
    }

    public fun burn(nft: NFT<AnyNFT, AnyNFTBody>) acquires AnyNFTBurnCapability{
        let cap = borrow_global_mut<AnyNFTBurnCapability>(@creator);
        let AnyNFTBody{} = NFT::burn_with_cap(&mut cap.cap, nft);
    }

    public fun do_accept(sender: &signer) {
        NFTGallery::accept<AnyNFT, AnyNFTBody>(sender);
    }

    public(script) fun accept(sender: signer) {
        Self::do_accept(&sender);
    }

    public(script) fun transfer(sender: signer, id: u64, receiver: address) {
        NFTGallery::transfer<AnyNFT, AnyNFTBody>(&sender, id, receiver);
    }
}

// check: EXECUTED

//# run --signers creator
script {
    use creator::AnyNFT;
    fun main(sender: signer) {
        AnyNFT::init(&sender);
    }
}

// check: EXECUTED

//# run --signers bob
script {
    use creator::AnyNFT;
    fun main(account: signer) {
        AnyNFT::accept(account);
    }
}

// check: EXECUTED


//# run --signers bob
script {
use StarcoinFramework::Option;
use creator::AnyNFT::{AnyNFT, AnyNFTBody};
use StarcoinFramework::NFTGallery;
use StarcoinFramework::Signer;
use StarcoinFramework::Vector;
fun main(sender: signer) {
    let sender_addr = Signer::address_of(&sender);
    let nft_info = NFTGallery::get_nft_info_by_id<AnyNFT, AnyNFTBody>(sender_addr, 1);
    assert!(Option::is_none(&nft_info), 1000);
    let nft_infos = NFTGallery::get_nft_infos<AnyNFT, AnyNFTBody>(sender_addr);
    assert!(Vector::is_empty(&nft_infos), 1001);
}
}

//# run --signers bob
script {
use creator::AnyNFT;
fun main(sender: signer) {
    AnyNFT::mint(&sender);
}
}

// check: EXECUTED

//# run --signers bob
script {
use StarcoinFramework::Option;
use creator::AnyNFT::{AnyNFT, AnyNFTBody};
use StarcoinFramework::NFTGallery;
use StarcoinFramework::Signer;
use StarcoinFramework::Vector;
fun main(sender: signer) {
    let sender_addr = Signer::address_of(&sender);
    let nft_info = NFTGallery::get_nft_info_by_id<AnyNFT, AnyNFTBody>(sender_addr, 1);
    assert!(Option::is_some(&nft_info), 1002);
    let nft_infos = NFTGallery::get_nft_infos<AnyNFT, AnyNFTBody>(sender_addr);
    assert!(Vector::length(&nft_infos) == 1, 1003);
}
}

// check: EXECUTED

//# run --signers bob --gas-budget 40000000
script {
use creator::AnyNFT;
fun main(sender: signer) {
    AnyNFT::mint_many(&sender, 200);
}
}

// check: EXECUTED



//# run --signers bob
script {
use creator::AnyNFT::{AnyNFT, AnyNFTBody};
use StarcoinFramework::NFTGallery;
use StarcoinFramework::Signer;
use StarcoinFramework::Vector;
fun main(sender: signer) {
    let sender_addr = Signer::address_of(&sender);
    let nft_infos = NFTGallery::get_nft_infos<AnyNFT, AnyNFTBody>(sender_addr);
    assert!(Vector::length(&nft_infos) == 201, 1004);
}
}

// check: EXECUTED


//# run --signers bob --gas-budget 40000000
script {
use creator::AnyNFT::{AnyNFT, AnyNFTBody};
use StarcoinFramework::NFTGallery;
use StarcoinFramework::Signer;
use StarcoinFramework::Option;
fun main(sender: signer) {
    let sender_addr = Signer::address_of(&sender);
    let id = 1;
    loop {
        //loop by id use more gas
        let info = NFTGallery::get_nft_info_by_id<AnyNFT, AnyNFTBody>(sender_addr, id);
        assert!(Option::is_some(&info), 1008);
        id = id + 1;
        if(id > 20){
            break
        }
    }
}
}

// check: EXECUTED

//# run --signers bob --gas-budget 40000000
script {
use creator::AnyNFT::{AnyNFT, AnyNFTBody};
use StarcoinFramework::NFTGallery;
use StarcoinFramework::Signer;
fun main(sender: signer) {
    let sender_addr = Signer::address_of(&sender);
    let idx = 0;
    loop {
        //loop by index
        let _info = NFTGallery::get_nft_info_by_idx<AnyNFT, AnyNFTBody>(sender_addr, idx);
        idx = idx + 1;
        if(idx >= 201){
            break
        }
    }
}
}

// check: EXECUTED

//# run --signers bob
script {
use creator::AnyNFT::{AnyNFT, AnyNFTBody};
use StarcoinFramework::NFTGallery;
use StarcoinFramework::Signer;
fun main(sender: signer) {
    let sender_addr = Signer::address_of(&sender);
    let count = NFTGallery::count_of<AnyNFT, AnyNFTBody>(sender_addr);
    assert!(count == 201, 1005);
}
}

// check: EXECUTED



//# run --signers bob
script {
use creator::AnyNFT::{Self, AnyNFT, AnyNFTBody};
use StarcoinFramework::NFTGallery;
fun main(sender: signer) {
    let nft = NFTGallery::withdraw_one<AnyNFT, AnyNFTBody>(&sender);
    AnyNFT::burn(nft);
}
}

// check: EXECUTED


//# run --signers bob
script {
use creator::AnyNFT::{Self, AnyNFT, AnyNFTBody};
use StarcoinFramework::NFTGallery;
use StarcoinFramework::Option;
fun main(sender: signer) {
    //withdraw by id  use more gas than withdraw one
    let nft = NFTGallery::withdraw<AnyNFT, AnyNFTBody>(&sender, 1);
    assert!(Option::is_some(&nft), 1007);
    let nft = Option::destroy_some(nft);
    AnyNFT::burn(nft);
}
}

// check: EXECUTED