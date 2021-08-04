//! account: creator
//! account: bob

//! sender: creator
address creator = {{creator}};
module creator::TestNFT {
    struct TestNFT has copy, store, drop{}

    public fun new(): TestNFT{
        TestNFT{}
    }
}

// check: EXECUTED

//! new-transaction
//! sender: creator
address creator = {{creator}};
script {
    use 0x1::NFT;
    use creator::TestNFT::{Self, TestNFT};
    use 0x1::NFTGallery;
    fun main(account: signer) {
        NFT::register<TestNFT>(&account);
        NFTGallery::accept<TestNFT>(&account);
        let metadata = NFT::new_meta_with_image(b"test_nft_1", b"ipfs:://xxxxxx", b"This is a test nft.");
        let nft = NFT::mint<TestNFT>(&account, metadata, TestNFT::new());
        NFTGallery::deposit(&account, nft);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
    use 0x1::NFTGallery;
    use creator::TestNFT::TestNFT;
    fun main(account: signer) {
        NFTGallery::accept<TestNFT>(&account);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: creator
//! args: {{bob}}
address creator = {{creator}};
script {
    use creator::TestNFT::TestNFT;
    use 0x1::NFTGallery;
    fun main(account: signer, address: address) {
        NFTGallery::transfer<TestNFT>(&account, 1, address);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: bob
address creator = {{creator}};
script {
use 0x1::Option;
use creator::TestNFT::TestNFT;
use 0x1::NFTGallery;
fun main(account: signer) {
    let nft = NFTGallery::get_nft_info<TestNFT>(&account, 1);
    assert(Option::is_some(&nft), 1000);
}
}