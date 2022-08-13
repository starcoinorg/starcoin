//# init --rpc http://halley.seed.starcoin.org  --block-number 81980

//# faucet --addr creator --amount 100000000000

//# call-api chain.get_block_by_number [81980]

//# run --signers creator --args {{$.call-api[0].header.number}}u64  --args {{$.call-api[0].header.block_hash}}
script{
    use StarcoinFramework::Vector;
    fun main(_sender: signer, block_number: u64, block_hash: vector<u8>){
        assert!(block_number == 81980, 1000);
        assert!(Vector::length(&block_hash) == 32, 1001);
        assert!(block_hash == x"5238815ba614545007739f15660735a1b8393fc29cd195a4471216589533dca1", 1002); // expect equal to block_hash on halley
    }
}

//# call-api state.get_with_proof_by_root_raw ["0x6bfb460477adf9dd0455d3de2fc7f211/1/0x00000000000000000000000000000001::IdentifierNFT::IdentifierNFT<0x6bfb460477adf9dd0455d3de2fc7f211::SBTModule::DaoMember<0x6bfb460477adf9dd0455d3de2fc7f211::SBTModule::SbtTestDAO>,0x6bfb460477adf9dd0455d3de2fc7f211::SBTModule::DaoMemberBody<0x6bfb460477adf9dd0455d3de2fc7f211::SBTModule::SbtTestDAO>>","0x7c0c3b5d037241d4487f4eaf77a56c3f9600441b7524df6b9ad3f34f91364ed2"]

//# run --signers creator --args {{$.call-api[1]}}
script{
    fun main(_sender: signer, snapshot_raw_proofs: vector<u8>){
        let expect_raw_proofs = x"0145016bfb460477adf9dd0455d3de2fc7f21101000000000000000664616f313031000a69616d67655f64617461005704000000000000640000000000000000000000000000000145020120cc969848619e507450ebf01437155ab5f2dbb554fe611cb71958855a1b2ec6640120e9036d04ecb8da65ec2f576e3d44a7e3b361615665528d5b3a24cd55d331c1be012073837fcf4e69ae60e18ea291b9f25c86ce8053c6ee647a2b287083327c67dd7e201e488652be3576bbdf5f767ac4c45ea65b626a7185af04c600f3a7b7582199cc0720bd651193de5be5fd19c40b9dbc26d52d22bc40b65f92112cb9dc69560b811a37205350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000205350415253455f4d45524b4c455f504c414345484f4c4445525f484153480000200911f753d151afd303488aa85cb246986fbf18cc055f29f5aa18e2159af38ac42053ef9425d82ac8137c11554fbde3cc938c6ee1302a8953f3a83ae97c33a5779b203ac06fbab3073f4c710197531e67e606998f31041c43c6b05becb7efb82c65e6204483b32b742a84982c2f97493f8eec4064f4e53d11c5f8103983993f0d350cb50120fa60f8311936961f5e9dee5ccafaea83ed91c6eaa04a7dea0b85a38cf84d8564207ef6a85019523861474cdf47f4db8087e5368171d95cc2c1e57055a72ca39cb70420ac2f136cddd462a3df5555a94ead3b1e09d401d768a36c625fc1e9da948f90fd205350415253455f4d45524b4c455f504c414345484f4c4445525f48415348000020284c597ee0618821ea22fecfd2e6da6f3e5a451ee13870e6338e840e50a025fb20e18365cf9b9eb20b0df4a5c08e94e648d8706860fd7891ca3742667a3e55690c";
        assert!(expect_raw_proofs == snapshot_raw_proofs, 2000);
    }
}

//# block --number 81981

//# call-api chain.get_block_by_number [81981]

//# run --signers creator --args {{$.call-api[2].header.number}}u64  --args {{$.call-api[2].header.block_hash}}
script{
    use StarcoinFramework::Vector;
    fun main(_sender: signer, block_number: u64, block_hash: vector<u8>){
        assert!(block_number == 81981, 3000);
        assert!(Vector::length(&block_hash) == 32, 3001);
        assert!(block_hash != x"b538880d77785ad847d85c8f7be39b059ec50ac7c3497fb964aca7ff78a03985", 3002); // expect not equal to block_hash on halley
    }
}