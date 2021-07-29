//! account: creator
//! account: bob

// check: EXECUTED

//! new-transaction
//! sender: genesis
script {
    use 0x1::NFT;
    fun main(account: signer) {
        NFT::initialize(&account);
    }
}


// check: EXECUTED

//! new-transaction
//! sender: creator
address creator = {{creator}};
module creator::TestNFT {
    struct TestNFT has store, key{}

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
        NFT::register_nft<TestNFT>(&account, 1024);
        NFTGallery::init<TestNFT>(&account);
        NFTGallery::create_nft<TestNFT>(&account, b"abc", TestNFT::new());
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
        NFTGallery::transfer_nft<TestNFT>(&account, 1, address);
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
    let nft = NFTGallery::get_nft<TestNFT>(&account, 1);
    assert(Option::is_some(&nft), 1000);
}
}