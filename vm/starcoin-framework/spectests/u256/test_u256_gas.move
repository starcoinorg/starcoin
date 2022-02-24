//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use StarcoinFramework::U256;
    fun main() {
        let _ = U256::from_u64(10);
    }
}

//# run --signers alice
script {
    use StarcoinFramework::U256;
    fun main() {
        let _ = U256::zero();
    }

}


//# run --signers alice
script {
    use StarcoinFramework::U256;
    fun main() {
        let _ = U256::from_u128(10u128);
    }

}
//# run --signers alice
script {
    use StarcoinFramework::U256;
    use StarcoinFramework::Vector;
    fun main() {
        let _ = U256::from_little_endian(Vector::singleton(1u8));
    }
}
//# run --signers alice
script {
    use StarcoinFramework::U256;
    fun main() {
        let a = U256::zero();
        let b = U256::one();

        let _ = U256::add(a, b);
    }
}
//# run --signers alice
script {
    use StarcoinFramework::U256;
    fun main() {
        let a = U256::zero();
        let b = U256::one();

        let _ = U256::sub(b, a);
    }
}
//# run --signers alice
script {
    use StarcoinFramework::U256;
    fun main() {
        let a = U256::zero();
        let b = U256::one();

        let _ = U256::mul(b, a);
    }
}
//# run --signers alice
script {
    use StarcoinFramework::U256;
    fun main() {
        let a = U256::one();
        let b = U256::one();

        let _ = U256::div(a, b);
    }
}

