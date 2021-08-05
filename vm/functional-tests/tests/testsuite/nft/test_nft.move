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
        NFT::register<TestNFT,NFTInfo>(sender,NFTInfo{});
        Self::do_accept(sender);
    }

    public fun mint(sender: &signer){
        let metadata = NFT::new_meta_with_image(b"test_nft_1", b"ipfs:://xxxxxx", b"This is a test nft.");
        let nft = NFT::mint<TestNFT,TestNFTBody>(sender, metadata, TestNFT{}, TestNFTBody{});
        NFTGallery::deposit(sender, nft);
    }

    public fun do_accept(sender: &signer) {
        NFTGallery::accept<TestNFT, TestNFTBody>(sender);
    }

    public(script) fun accept(sender: signer) {
        Self::do_accept(&sender);
    }

    public(script) fun transfer(sender: signer, uid: u64, receiver: address) {
        NFTGallery::transfer<TestNFT, TestNFTBody>(&sender, uid, receiver);
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
    use creator::TestNFT;
    fun main(account: signer) {
        TestNFT::accept(account);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: creator
//! args: {{bob}}
address creator = {{creator}};
script {
    use creator::TestNFT;
    fun main(sender: signer, receiver: address) {
        TestNFT::transfer(sender, 1, receiver);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
use 0x1::Option;
use creator::TestNFT::{TestNFT, TestNFTBody};
use 0x1::NFTGallery;
fun main(account: signer) {
    let nft = NFTGallery::get_nft_info<TestNFT, TestNFTBody>(&account, 1);
    assert(Option::is_some(&nft), 1000);
}
}