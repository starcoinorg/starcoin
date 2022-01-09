address StarcoinFramework {
/// Functions for bit operations.
module BitOperators {

    spec module {
        pragma verify = false;
    }

    /// bit and: x & y
    public fun and(x: u64, y: u64): u64 {
        (x & y as u64)
    }

    /// bit or: x | y
    public fun or(x: u64, y: u64): u64 {
        (x | y as u64)
    }

    /// bit xor: x ^ y
    public fun xor(x: u64, y: u64): u64 {
        (x ^ y as u64)
    }

    /// bit not: !x
    public fun not(x: u64): u64 {
       (x ^ 18446744073709551615u64 as u64)
    }

    /// left shift n bits.
    public fun lshift(x: u64, n: u8): u64 {
        (x << n  as u64)
    }

    /// right shift n bits.
    public fun rshift(x: u64, n: u8): u64 {
        (x >> n  as u64)
    }
}
}