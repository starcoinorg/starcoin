//# init -n dev


//# faucet --addr alice

//# run --signers alice
script {
    use starcoin_framework::Vector;
    use starcoin_framework::signer;
    use starcoin_framework::Option;
    use starcoin_framework::NFT;
    use starcoin_framework::GenesisNFT::{Self, GenesisNFTMeta};
    //TODO: generate the real root and proof
    fun main(sender: signer) {
        let proof = Vector::empty<vector<u8>>();
        GenesisNFT::mint(&sender, 1, proof);
        let info = GenesisNFT::get_info(signer::address_of(&sender));
        assert!(Option::is_some<NFT::NFTInfo<GenesisNFTMeta>>(&info), 1000);
    }
}

// check:  VMExecutionFailure(ABORTED { code: 256511
