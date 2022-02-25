address StarcoinFramework {
/// Implementation of i64.
module SignedInteger64 {

    /// Define a signed integer type with two 32 bits.
    struct SignedInteger64 has copy, drop, store {
        value: u64,
        is_negative: bool,
    }

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    /// Multiply a u64 integer by a signed integer number.
    public fun multiply_u64(num: u64, multiplier: SignedInteger64): SignedInteger64 {
        let product = multiplier.value * num;
        SignedInteger64 { value: (product as u64), is_negative: multiplier.is_negative }
    }

    /// Divide a u64 integer by a signed integer number.
    public fun divide_u64(num: u64, divisor: SignedInteger64): SignedInteger64 {
        let quotient = num / divisor.value;
        SignedInteger64 { value: (quotient as u64), is_negative: divisor.is_negative }
    }

    /// Sub: `num - minus`
    public fun sub_u64(num: u64, minus: SignedInteger64): SignedInteger64 {
        if (minus.is_negative) {
            let result = num + minus.value;
            SignedInteger64 { value: (result as u64), is_negative: false }
        } else {
            if (num > minus.value)  {
                let result = num - minus.value;
                SignedInteger64 { value: (result as u64), is_negative: false }
            }else {
                let result = minus.value - num;
                SignedInteger64 { value: (result as u64), is_negative: true }
            }
        }
    }
    /// Add: `num + addend`
    public fun add_u64(num: u64, addend: SignedInteger64): SignedInteger64 {
        if (addend.is_negative) {
           if (num > addend.value)  {
               let result = num - addend.value;
               SignedInteger64 { value: (result as u64), is_negative: false }
           }else {
               let result = addend.value - num;
               SignedInteger64 { value: (result as u64), is_negative: true }
           }
        } else {
             let result = num + addend.value;
             SignedInteger64 { value: (result as u64), is_negative: false }
        }
    }

    /// Create a signed integer value from a unsigned integer
    public fun create_from_raw_value(value: u64, is_negative: bool): SignedInteger64 {
        SignedInteger64 { value, is_negative }
    }

    /// Get value part of i64 ignore sign part.
    public fun get_value(num: SignedInteger64): u64 {
        num.value
    }

    /// Check if the given num is negative.
    public fun is_negative(num: SignedInteger64): bool {
        num.is_negative
    }

    // **************** SPECIFICATIONS ****************

    

    spec multiply_u64 {
       aborts_if multiplier.value * num > max_u64();
    }

    spec divide_u64 {
        aborts_if divisor.value == 0;
    }

    spec sub_u64 {
        aborts_if minus.is_negative && num + minus.value > max_u64();
    }

    spec add_u64 {
       aborts_if !addend.is_negative && num + addend.value > max_u64();
    }

    spec create_from_raw_value {
        aborts_if false;
        ensures result == SignedInteger64 { value, is_negative };
    }

    spec get_value {
        aborts_if false;
        ensures result == num.value;
    }

    spec is_negative {
        aborts_if false;
        ensures result == num.is_negative;
    }
}

}
