//# init -n dev

//# faucet --addr alice


//# run --signers alice
// test bitwise operator
script {
    use StarcoinFramework::BitOperators::{and, or, xor, not, lshift, rshift};
fun main() {
    assert!(and(0u64, 0u64) == 0u64, 2000);
    assert!(or(0u64, 0u64) == 0u64, 2001);
    assert!(xor(0u64, 0u64) == 0u64, 2002);
    assert!(and(0u64, 1u64) == 0u64, 2003);
    assert!(or(1u64, 2u64) == 3u64, 2004);
    assert!(xor(3u64, 6u64) == 5u64, 2005);
    assert!(lshift(0u64, 0u8) == 0u64, 2006);
    assert!(rshift(0u64, 0u8) == 0u64, 2007);
    assert!(lshift(1u64, 2u8) == 4u64, 2008);
    assert!(rshift(8u64, 2u8) == 2u64, 2009);
    assert!(rshift(8u64, 3u8) == 1u64, 2010);
    assert!(rshift(8u64, 4u8) == 0u64, 2011);
    assert!(not(0u64) == 18446744073709551615u64, 2012);
    assert!(not(1u64) == 18446744073709551614u64, 2013);
}
}

//# run --signers alice

// test bit operator overflow

script {
    use StarcoinFramework::BitOperators::{and, or, xor, lshift, rshift};
fun main() {
    assert!(and(18446744073709551615u64, 18446744073709551615u64) == 18446744073709551615u64, 1101);
    assert!(or(18446744073709551615u64, 18446744073709551615u64) == 18446744073709551615u64, 1102);
    assert!(xor(18446744073709551615u64, 1u64) == 18446744073709551614u64, 1103);
    assert!(xor(18446744073709551615u64, 18446744073709551615u64) == 0u64, 1103);
    assert!(lshift(18446744073709551615u64, 63u8) == 9223372036854775808u64, 1104);
    assert!(rshift(18446744073709551615u64, 63u8) == 1u64, 1106);
}

}
