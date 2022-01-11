//# init -n dev

//# faucet --addr creator

//# faucet --addr bob


//#publish
module creator::TestNFT {
    use StarcoinFramework::NFT;
    use StarcoinFramework::NFTGallery;
    struct TestNFT has copy, store, drop{}
    struct TestNFTBody has store{
    }
    public fun init(sender: &signer){
        NFT::register_v2<TestNFT>(sender, NFT::empty_meta());
        Self::do_accept(sender);
    }

    public fun mint(sender: &signer){
        let metadata = NFT::new_meta_with_image(b"test_nft_1", b"ipfs:://xxxxxx", b"This is a test nft.");
        let nft = NFT::mint_v2<TestNFT,TestNFTBody>(sender, metadata, TestNFT{}, TestNFTBody{});
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

//# run --signers creator
script {
    use StarcoinFramework::NFT;
    use creator::TestNFT::{Self, TestNFT};
    fun main(sender: signer) {
        TestNFT::init(&sender);
        assert!(NFT::is_register<TestNFT>(), 1001);
        TestNFT::mint(&sender);
    }
}

// check: EXECUTED

//# run --signers bob
script {
    use creator::TestNFT::{TestNFT,TestNFTBody};
    use StarcoinFramework::NFTGalleryScripts;
    fun main(sender: signer) {
        NFTGalleryScripts::accept<TestNFT,TestNFTBody>(sender);
    }
}

// check: gas_used
// check: 97398
// check: EXECUTED

//# run --signers creator --args @bob
script {
    use creator::TestNFT::{TestNFT,TestNFTBody};
    use StarcoinFramework::NFTGalleryScripts;
    fun main(sender: signer, receiver: address) {
        NFTGalleryScripts::transfer<TestNFT,TestNFTBody>(sender, 1, receiver);
    }
}

// check: gas_used
// check: 219220
// check: EXECUTED

//# run --signers bob
script {
use StarcoinFramework::Option;
use creator::TestNFT::{TestNFT, TestNFTBody};
use StarcoinFramework::NFTGallery;
use StarcoinFramework::Signer;
fun main(sender: signer) {
    let sender_addr = Signer::address_of(&sender);
    let nft = NFTGallery::get_nft_info_by_id<TestNFT, TestNFTBody>(sender_addr, 1);
    assert!(Option::is_some(&nft), 1000);
}
}