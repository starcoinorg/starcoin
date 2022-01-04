//# init -n dev

//# faucet --addr creator

//# run --signers creator
script {
    use Std::Math;
    use Std::Debug;
    fun main(_signer: signer) {
        Debug::print(&Math::u64_max());
        Debug::print(&Math::u128_max());
    }
}
