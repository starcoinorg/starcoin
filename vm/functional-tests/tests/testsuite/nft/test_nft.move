//! account: creator
//! account: bob

//! sender: creator
address creator = {{creator}};
module creator::TestNFT {
    use 0x1::NFT;
    use 0x1::NFTGallery;
    struct TestNFT has copy, store, drop{}
    struct TestNFTBody has store{
    }
    struct NFTInfo has copy,store,drop{}
    public fun init(sender: &signer){
        let nft_type_info=NFT::new_nft_type_info(sender, NFTInfo{}, NFT::empty_meta());
        NFT::register<TestNFT,NFTInfo>(sender, nft_type_info);
        Self::do_accept(sender);
    }

    public fun mint(sender: &signer){
        let metadata = NFT::new_meta_with_image(b"test_nft_1", b"ipfs:://xxxxxx", b"This is a test nft.");
        let nft = NFT::mint<TestNFT,TestNFTBody,NFTInfo>(sender, metadata, TestNFT{}, TestNFTBody{});
        NFTGallery::deposit(sender, nft);
    }

    public fun do_accept(sender: &signer) {
        NFTGallery::accept<TestNFT, TestNFTBody>(sender);
    }

    public(script) fun accept(sender: signer) {
        Self::do_accept(&sender);
    }

    public(script) fun transfer(sender: signer, id: u64, receiver: address) {
        NFTGallery::transfer<TestNFT, TestNFTBody>(&sender, id, receiver);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: creator
address creator = {{creator}};
script {
    use creator::TestNFT;
    fun main(sender: signer) {
        TestNFT::init(&sender);
        TestNFT::mint(&sender);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use creator::TestNFT::{TestNFT,TestNFTBody};
    use 0x1::NFTGalleryScripts;
    fun main(sender: signer) {
        NFTGalleryScripts::accept<TestNFT,TestNFTBody>(sender);
    }
}

// check: gas_used
// check: 492608
// check: EXECUTED

//! new-transaction
//! sender: creator
//! args: {{bob}}
address creator = {{creator}};
script {
    use creator::TestNFT::{TestNFT,TestNFTBody};
    use 0x1::NFTGalleryScripts;
    fun main(sender: signer, receiver: address) {
        NFTGalleryScripts::transfer<TestNFT,TestNFTBody>(sender, 1, receiver);
    }
}

// check: gas_used
// check: 218929
// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
use 0x1::Option;
use creator::TestNFT::{TestNFT, TestNFTBody};
use 0x1::NFTGallery;
use 0x1::Signer;
fun main(sender: signer) {
    let sender_addr = Signer::address_of(&sender);
    let nft = NFTGallery::get_nft_info_by_id<TestNFT, TestNFTBody>(sender_addr, 1);
    assert(Option::is_some(&nft), 1000);
}
}