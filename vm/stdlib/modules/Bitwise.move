address 0x1 {
module BitOperators {

    spec module {
        pragma verify = false;
    }

    public fun and(x: u64, y: u64): u64 {
        (x & y as u64)
    }

    public fun or(x: u64, y: u64): u64 {
        (x | y as u64)
    }

    public fun xor(x: u64, y: u64): u64 {
        (x ^ y as u64)
    }

    public fun not(x: u64): u64 {
       (x ^ 18446744073709551615u64 as u64)
    }

    public fun lshift(x: u64, n: u8): u64 {
        (x << n  as u64)
    }

    public fun rshift(x: u64, n: u8): u64 {
        (x >> n  as u64)
    }
}
}