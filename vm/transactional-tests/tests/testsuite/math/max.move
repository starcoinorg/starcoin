//# init -n dev

//# faucet --addr creator

//# run --signers creator
script {
    use StarcoinFramework::Math;
    use StarcoinFramework::Debug;
    fun main(_signer: signer) {
        Debug::print(&Math::u64_max());
        Debug::print(&Math::u128_max());
    }
}
