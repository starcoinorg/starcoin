address StarcoinFramework {
/// The module provide operations for FixedPoint32.
module FixedPoint32 {

    use StarcoinFramework::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// Define a fixed-point numeric type with 32 fractional bits.
    /// This is just a u64 integer but it is wrapped in a struct to
    /// make a unique type.
    struct FixedPoint32 has copy, drop, store { value: u64 }

    const MAX_U64: u128 = 18446744073709551615;

    /// The denominator provided was zero
    const EDENOMINATOR: u64 = 101;
    /// The quotient value would be too large to be held in a `u64`
    const EDIVISION: u64 = 102;
    /// The multiplied value would be too large to be held in a `u64`
    const EMULTIPLICATION: u64 = 103;
    /// A division by zero was encountered
    const EDIVISION_BY_ZERO: u64 = 104;
    /// The computed ratio when converting to a `FixedPoint32` would be unrepresentable
    const ERATIO_OUT_OF_RANGE: u64 = 105;

    /// Multiply a u64 integer by a fixed-point number, truncating any
    /// fractional part of the product. This will abort if the product
    /// overflows.
    public fun multiply_u64(val: u64, multiplier: FixedPoint32): u64 {
        // The product of two 64 bit values has 128 bits, so perform the
        // multiplication with u128 types and keep the full 128 bit product
        // to avoid losing accuracy.
        let unscaled_product = (val as u128) * (multiplier.value as u128);
        // The unscaled product has 32 fractional bits (from the multiplier)
        // so rescale it by shifting away the low bits.
        let product = unscaled_product >> 32;
        // Check whether the value is too large.
        assert!(product <= MAX_U64, Errors::limit_exceeded(EMULTIPLICATION));
        (product as u64)
    }
    spec multiply_u64 {
        /// Currently, we ignore the actual implementation of this function in verification
        /// and treat it as uninterpreted, which simplifies the verification problem significantly.
        /// This way we avoid the non-linear arithmetic problem presented by this function.
        ///
        /// Abstracting this and related functions is possible because the correctness of currency
        /// conversion (where `FixedPoint32` is used for) is not relevant for the rest of the contract
        /// control flow, so we can assume some arbitrary (but fixed) behavior here.
        pragma opaque = true;
        pragma verify = false;
        ensures result == spec_multiply_u64(val, multiplier);
    }

    /// Divide a u64 integer by a fixed-point number, truncating any
    /// fractional part of the quotient. This will abort if the divisor
    /// is zero or if the quotient overflows.
    public fun divide_u64(val: u64, divisor: FixedPoint32): u64 {
        // Check for division by zero.
        assert!(divisor.value != 0, Errors::invalid_argument(EDIVISION_BY_ZERO));
        // First convert to 128 bits and then shift left to
        // add 32 fractional zero bits to the dividend.
        let scaled_value = (val as u128) << 32;
        let quotient = scaled_value / (divisor.value as u128);
        // Check whether the value is too large.
        assert!(quotient <= MAX_U64, Errors::limit_exceeded(EDIVISION));
        // the value may be too large, which will cause the cast to fail
        // with an arithmetic error.
        (quotient as u64)
    }
    spec divide_u64 {
        /// See comment at `Self::multiply_64`.
        pragma opaque = true;
        pragma verify = false;
        ensures result == spec_divide_u64(val, divisor);
    }

    /// Create a fixed-point value from a rational number specified by its
    /// numerator and denominator. This function is for convenience; it is also
    /// perfectly fine to create a fixed-point value by directly specifying the
    /// raw value. This will abort if the denominator is zero or if the ratio is
    /// not in the range 2^-32 .. 2^32-1.
    public fun create_from_rational(numerator: u64, denominator: u64): FixedPoint32 {
        // If the denominator is zero, this will abort.
        // Scale the numerator to have 64 fractional bits and the denominator
        // to have 32 fractional bits, so that the quotient will have 32
        // fractional bits.
        let scaled_numerator = (numerator as u128) << 64;
        let scaled_denominator = (denominator as u128) << 32;
        assert!(scaled_denominator != 0, Errors::invalid_argument(EDENOMINATOR));
        let quotient = scaled_numerator / scaled_denominator;
        assert!(quotient != 0 || numerator == 0, Errors::invalid_argument(ERATIO_OUT_OF_RANGE));
        // Return the quotient as a fixed-point number. We first need to check whether the cast
        // can succeed.
        assert!(quotient <= MAX_U64, Errors::limit_exceeded(ERATIO_OUT_OF_RANGE));
        FixedPoint32 { value: (quotient as u64) }
    }
    spec create_from_rational {
        /// See comment at `Self::multiply_64`.
        pragma opaque = true;
        pragma verify = false;
        ensures result == spec_create_from_rational(numerator, denominator);
    }

    /// create a fixedpoint 32  from u64.
    public fun create_from_raw_value(value: u64): FixedPoint32 {
        FixedPoint32 { value }
    }

    /// Accessor for the raw u64 value. Other less common operations, such as
    /// adding or subtracting FixedPoint32 values, can be done using the raw
    /// values directly.
    public fun get_raw_value(num: FixedPoint32): u64 {
        num.value
    }

    // **************** SPECIFICATIONS ****************


        /// Uninterpreted function for `Self::multiply_u64`.
        spec fun spec_multiply_u64(val: u64, multiplier: FixedPoint32): u64;

        /// Uninterpreted function for `Self::divide_u64`.
        spec fun spec_divide_u64(val: u64, divisor: FixedPoint32): u64;

        /// Uninterpreted function for `Self::create_from_rational`.
        spec fun spec_create_from_rational(numerator: u64, denominator: u64): FixedPoint32;


}

}
