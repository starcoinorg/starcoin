module 0x1::U256 {

    use 0x1::Vector;

    const WORD: u8 = 4;


    /// use vector to represent data.
    /// so that we can use buildin vector ops later to construct U256.
    /// vector should always has two elements.
    struct U256 has copy, drop, store {
        /// little endian representation
        bits: vector<u64>,
    }

    public fun zero(): U256 {
        from_u128(0u128)
    }
    public fun one(): U256 {
        from_u128(1u128)
    }

    public fun from_u64(v: u64): U256 {
        from_u128((v as u128))
    }

    public fun from_u128(v: u128): U256 {
        let low = ( (v & 0xffffffffffffffff) as u64);
        let high = ((v >> 64) as u64);
        let bits = Vector::singleton(low);
        Vector::push_back(&mut bits, high);
        Vector::push_back(&mut bits, 0u64);
        Vector::push_back(&mut bits, 0u64);
        U256 {
            bits
        }
    }

    #[test]
    fun test_from_u128() {
        // 2^64 + 1
        let v = from_u128(18446744073709551617u128);
        assert(*Vector::borrow(&v.bits, 0) == 1, 0);
        assert(*Vector::borrow(&v.bits, 1) == 1, 1);
        assert(*Vector::borrow(&v.bits, 2) == 0, 2);
        assert(*Vector::borrow(&v.bits, 3) == 0, 3);
    }

    public fun from_big_endian(data: vector<u8>): U256 {
        // TODO: define error code.
        assert(Vector::length(&data) <= 32, 4040);
        from_bytes(data, true)
    }
    public fun from_little_endian(data: vector<u8>): U256 {
        // TODO: define error code.
        assert(Vector::length(&data) <= 32, 4040);
        from_bytes(data, false)
    }


    const EQUAL: u8 = 0;
    const LESS_THAN: u8 = 1;
    const GREATER_THAN: u8 = 2;

    public fun compare(a: &U256, b: &U256): u8 {
        let i = (WORD as u64);
        while (i > 0) {
            i = i - 1;
            let a_bits = *Vector::borrow(&a.bits, i);
            let b_bits = *Vector::borrow(&b.bits, i);
            if (a_bits != b_bits) {
                if (a_bits < b_bits) {
                    return LESS_THAN
                } else {
                    return GREATER_THAN
                }
            }
        };
        EQUAL
    }

    #[test]
    fun test_compare() {
        let a = from_u64(111);
        let b = from_u64(111);
        let c = from_u64(112);
        let d = from_u64(110);
        assert(compare(&a, &b) == EQUAL, 0);
        assert(compare(&a, &c) == LESS_THAN, 1);
        assert(compare(&a, &d) == GREATER_THAN, 2);
    }

    public fun add(a: U256, b: U256): U256 {
        native_add(&mut a, &b);
        a
    }

    #[test]
    fun test_add() {
        let a = Self::one();
        let b = Self::from_u128(10);
        let ret = Self::add(a, b);
        assert(compare(&ret, &from_u64(11)) == EQUAL, 0);
    }

    public fun sub(a: U256, b: U256): U256 {
        native_sub(&mut a, &b);
        a
    }
    #[test]
    #[expected_failure]
    fun test_sub_overflow() {
        let a = Self::one();
        let b = Self::from_u128(10);
        let _ = Self::sub(a, b);
    }
    #[test]
    fun test_sub_ok() {
        let a = Self::from_u128(10);
        let b = Self::one();
        let ret = Self::sub(a, b);
        assert(compare(&ret, &from_u64(9)) == EQUAL, 0);
    }

    public fun mul(a: U256, b: U256): U256 {
        native_mul(&mut a, &b);
        a
    }

    #[test]
    fun test_mul() {
        let a = Self::from_u128(10);
        let b = Self::from_u64(10);
        let ret = Self::mul(a, b);
        assert(compare(&ret, &from_u64(100)) == EQUAL, 0);
    }

    public fun div(a: U256, b: U256): U256 {
        native_div(&mut a, &b);
        a
    }
    #[test]
    fun test_div() {
        let a = Self::from_u128(10);
        let b = Self::from_u64(2);
        let c = Self::from_u64(3);
        assert(compare(&Self::div(a, b), &from_u64(5)) == EQUAL, 0);
        assert(compare(&Self::div(a, c), &from_u64(3)) == EQUAL, 0);
    }

    public fun rem(a: U256, b:U256): U256 {
        native_rem(&mut a, &b);
        a
    }
    #[test]
    fun test_rem() {
        let a = Self::from_u128(10);
        let b = Self::from_u64(2);
        let c = Self::from_u64(3);
        assert(compare(&Self::rem(a, b), &from_u64(0)) == EQUAL, 0);
        assert(compare(&Self::div(a, c), &from_u64(1)) == EQUAL, 0);
    }

    native fun from_bytes(data: vector<u8>, be: bool): U256;
    native fun native_add(a: &mut U256, b: &U256);
    native fun native_sub(a: &mut U256, b: &U256);
    native fun native_mul(a: &mut U256, b: &U256);
    native fun native_div(a: &mut U256, b: &U256);
    native fun native_rem(a: &mut U256, b: &U256);
    native fun native_pow(a: &mut U256, b: &U256);
}
