script {
    use 0x1::Math;
    use 0x1::Debug;
    fun main(_signer: &signer) {
        Debug::print(&Math::u64_max());
        Debug::print(&Math::u128_max());
    }
}
