//# init -n dev


//# faucet --addr alice

//# run --signers alice
script {
    use StarcoinFramework::Vector;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Option;
    use StarcoinFramework::NFT;
    use StarcoinFramework::GenesisNFT::{Self, GenesisNFTMeta};
    //TODO: generate the real root and proof
    fun main(sender: signer) {
        let proof = Vector::empty<vector<u8>>();
        GenesisNFT::mint(&sender, 1, proof);
        let info = GenesisNFT::get_info(Signer::address_of(&sender));
        assert!(Option::is_some<NFT::NFTInfo<GenesisNFTMeta>>(&info), 1000);
    }
}

// check:  VMExecutionFailure(ABORTED { code: 256511
