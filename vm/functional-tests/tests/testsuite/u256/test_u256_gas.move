
script {
    use 0x1::U256;
    fun main() {
        let _ = U256::from_u64(10);
    }
}

//check: "gas_used: 31743"

//! new-transaction

script {
    use 0x1::U256;
    fun main() {
        let _ = U256::zero();
    }

}


//check: "gas_used: 31709"

//! new-transaction

script {
    use 0x1::U256;
    fun main() {
        let _ = U256::from_u128(10u128);
    }

}
// check: "gas_used: 28804"

//! new-transaction

script {
    use 0x1::U256;
    use 0x1::Vector;
    fun main() {
        let _ = U256::from_little_endian(Vector::singleton(1u8));
    }
}
// check: "gas_used: 24514"

//! new-transaction
script {
    use 0x1::U256;
    fun main() {
        let a = U256::zero();
        let b = U256::one();

        let _ = U256::add(a, b);
    }
}
// check: gas_used
// check: 182112



//! new-transaction
script {
    use 0x1::U256;
    fun main() {
        let a = U256::zero();
        let b = U256::one();

        let _ = U256::sub(b, a);
    }
}
// check: gas_used
// check: 183488




//! new-transaction
script {
    use 0x1::U256;
    fun main() {
        let a = U256::zero();
        let b = U256::one();

        let _ = U256::mul(b, a);
    }
}
// check: gas_used
// check: 62764


//! new-transaction
script {
    use 0x1::U256;
    fun main() {
        let a = U256::one();
        let b = U256::one();

        let _ = U256::div(a, b);
    }
}
// check: gas_used
// check: 62768


