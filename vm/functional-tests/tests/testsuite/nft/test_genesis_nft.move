//! account: alice

//! new-transaction
//! sender: genesis
script {
    use 0x1::GenesisNFT;
    fun main(sender: signer) {
        let root = b"";
        GenesisNFT::initialize(&sender, root, 2, b"ipfs://xxx");
    }
}


// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::Vector;
    use 0x1::Signer;
    use 0x1::Option;
    use 0x1::NFT;
    use 0x1::GenesisNFT::{Self, GenesisNFTMeta};
    //TODO: generate the real root and proof
    fun main(sender: signer) {
        let proof = Vector::empty<vector<u8>>();
        GenesisNFT::mint(&sender, 1, proof);
        let info = GenesisNFT::get_info(Signer::address_of(&sender));
        assert(Option::is_some<NFT::NFTInfo<GenesisNFTMeta>>(&info), 1000);
    }
}

// check:  VMExecutionFailure(ABORTED { code: 256511
