//# init -n dev

//# faucet --addr creator

//# run --signers creator
script {
    use starcoin_framework::Math;
    use starcoin_framework::Debug;
    fun main(_signer: signer) {
        Debug::print(&Math::u64_max());
        Debug::print(&Math::u128_max());
    }
}
