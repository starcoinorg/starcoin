address StarcoinFramework {
/// The module provide some improved math calculations.
module Math {
    use StarcoinFramework::Vector;

    // TODO: verify the module.
    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict;
    }

    const U64_MAX:u64 = 18446744073709551615;
    const U128_MAX:u128 = 340282366920938463463374607431768211455;

    /// u64::MAX
    public fun u64_max(): u64 {
        U64_MAX
    }

    /// u128::MAX
    public fun u128_max(): u128 {
        U128_MAX
    }

    /// babylonian method (https://en.wikipedia.org/wiki/Methods_of_computing_square_roots#Babylonian_method)
    public fun sqrt(y: u128): u64 {
        if (y < 4) {
            if (y == 0) {
                0u64
            } else {
                1u64
            }
        } else {
            let z = y;
            let x = y / 2 + 1;
            while (x < z) {
                z = x;
                x = (y / x + x) / 2;
            };
            (z as u64)
        }
    }

    spec sqrt {
        pragma opaque = true;
        pragma verify = false; //while loop
        aborts_if [abstract] false;
        ensures [abstract] result == spec_sqrt();
    }

    /// We use an uninterpreted function to represent the result of sqrt. The actual value
    /// does not matter for the verification of callers.
    spec fun spec_sqrt(): u128;

    /// calculate the `y` pow of `x`.
    public fun pow(x: u64, y: u64): u128 {
        let result = 1u128;
        let z = y;
        let u = (x as u128);
        while (z > 0) {
            if (z % 2 == 1) {
                result = (u * result as u128);
            };
            u = (u * u as u128);
            z = z / 2;
        };
        result
    }

    spec pow {
        pragma opaque = true;
        pragma verify = false; //while loop
        aborts_if [abstract] false;
        ensures [abstract] result == spec_pow();
    }

    /// We use an uninterpreted function to represent the result of pow. The actual value
    /// does not matter for the verification of callers.
    spec fun spec_pow(): u128;

    /// https://medium.com/coinmonks/math-in-solidity-part-3-percents-and-proportions-4db014e080b1
    /// calculate x * y /z with as little loss of precision as possible and avoid overflow
    public fun mul_div(x: u128, y: u128, z: u128): u128 {
        if (y == z) {
            return x
        };
        if (x == z) {
            return y
        };
        let a = x / z;
        let b = x % z;
        //x = a * z + b;
        let c = y / z;
        let d = y % z;
        //y = c * z + d;
        a * c * z + a * d + b * c + b * d / z
    }

    spec mul_div {
        pragma opaque = true;
        include MulDivAbortsIf;
        aborts_if [abstract] false;
        ensures [abstract] result == spec_mul_div();
    }

    spec schema MulDivAbortsIf {
        x: u128;
        y: u128;
        z: u128;
        aborts_if y != z && x > z && z == 0;
        aborts_if y != z && x > z && z!=0 && x/z*y > MAX_U128;
        aborts_if y != z && x <= z && z == 0;
        //a * b overflow
        aborts_if y != z && x <= z && x / z * (x % z) > MAX_U128;
        //a * b * z overflow
        aborts_if y != z && x <= z && x / z * (x % z) * z > MAX_U128;
        //a * d overflow
        aborts_if y != z && x <= z && x / z * (y % z) > MAX_U128;
        //a * b * z + a * d overflow
        aborts_if y != z && x <= z && x / z * (x % z) * z + x / z * (y % z) > MAX_U128;
        //b * c overflow
        aborts_if y != z && x <= z && x % z * (y / z) > MAX_U128;
        //b * d overflow
        aborts_if y != z && x <= z && x % z * (y % z) > MAX_U128;
        //b * d / z overflow
        aborts_if y != z && x <= z && x % z * (y % z) / z > MAX_U128;
        //a * b * z + a * d + b * c overflow
        aborts_if y != z && x <= z && x / z * (x % z) * z + x / z * (y % z) + x % z * (y / z) > MAX_U128;
        //a * b * z + a * d + b * c + b * d / z overflow
        aborts_if y != z && x <= z && x / z * (x % z) * z + x / z * (y % z) + x % z * (y / z) + x % z * (y % z) / z > MAX_U128;

    }

    spec fun spec_mul_div(): u128;

    /// calculate sum of nums
    public fun sum(nums: &vector<u128>): u128 {
        let len = Vector::length(nums);
        let i = 0;
        let sum = 0;
        while (i < len){
            sum = sum + *Vector::borrow(nums, i);
            i = i + 1;
        };
        sum
    }

    /// calculate average of nums
    public fun avg(nums: &vector<u128>): u128{
        let len = Vector::length(nums);
        let sum = sum(nums);
        sum/(len as u128)
    }
}
}