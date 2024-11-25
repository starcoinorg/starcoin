//# init -n dev

//# faucet --addr alice --amount 100000000000000000

//# run --signers alice
script {
    use std::fixed_point32;

    fun main() {
        let f1 = fixed_point32::create_from_rational(3, 4); // 0.75
        let nine = fixed_point32::multiply_u64(12, copy f1); // 12 * 0.75
        assert!(nine == 9, nine);
        let twelve = fixed_point32::divide_u64(9, copy f1); // 9 / 0.75
        assert!(twelve == 12, twelve);

        let f2 = fixed_point32::create_from_rational(1, 3); // 0.333...
        let not_three = fixed_point32::multiply_u64(9, copy f2); // 9 * 0.333...
        // multiply_u64 does NOT round -- it truncates -- so values that
        // are not perfectly representable in binary may be off by one.
        assert!(not_three == 2, not_three);

        // Try again with a fraction slightly larger than 1/3.
        let f3 = fixed_point32::create_from_raw_value(fixed_point32::get_raw_value(copy f2) + 1);
        let three = fixed_point32::multiply_u64(9, copy f3);
        assert!(three == 3, three);

        // Test creating a 1.0 fraction from the maximum u64 value.
        let f4 = fixed_point32::create_from_rational(18446744073709551615, 18446744073709551615);
        let one = fixed_point32::get_raw_value(copy f4);
        assert!(one == 4294967296, 4); // 0x1.00000000
    }
}
