spec starcoin_std::math128 {

    spec max(a: u128, b: u128): u128 {
        aborts_if false;
        ensures a >= b ==> result == a;
        ensures a < b ==> result == b;
    }

    spec min(a: u128, b: u128): u128 {
        aborts_if false;
        ensures a < b ==> result == a;
        ensures a >= b ==> result == b;
    }

    spec average(a: u128, b: u128): u128 {
        pragma opaque;
        aborts_if false;
        ensures result == (a + b) / 2;
    }

    spec clamp(x: u128, lower: u128, upper: u128): u128 {
        requires (lower <= upper);
        aborts_if false;
        ensures (lower <=x && x <= upper) ==> result == x;
        ensures (x < lower) ==> result == lower;
        ensures (upper < x) ==> result == upper;
    }

    // The specs of `pow`, `floor_log2` and `sqrt` are validated with a smaller domain
    // in starcoin-core/third_party/move/move-prover/tests/sources/functional/math8.move

    spec pow(n: u128, e: u128): u128 {
        pragma opaque;
        aborts_if [abstract] spec_pow(n, e) > MAX_U128;
        ensures [abstract] result == spec_pow(n, e);
    }

    spec floor_log2(x: u128): u8 {
        pragma opaque;
        aborts_if [abstract] x == 0;
        ensures [abstract] spec_pow(2, result) <= x;
        ensures [abstract] x < spec_pow(2, result+1);
    }

    spec sqrt(x: u128): u128 {
        pragma opaque;
        aborts_if [abstract] false;
        ensures [abstract] x > 0 ==> result * result <= x;
        ensures [abstract] x > 0 ==> x < (result+1) * (result+1);
    }

    spec fun spec_pow(n: u128, e: u128): u128 {
        if (e == 0) {
            1
        }
        else {
            n * spec_pow(n, e-1)
        }
    }

    // spec mul_div(a: u128, b: u128, c: u128): u128 {
    //     pragma opaque = true;
    //     include MulDivAbortsIf;
    //     aborts_if [abstract] false;
    //     ensures [abstract] result == spec_mul_div();
    // }

    spec schema MulDivAbortsIf {
        a: u128;
        b: u128;
        c: u128;
        aborts_if b != c && a > c && c == 0;
        aborts_if b != c && a > c && c!=0 && a/c*b > MAX_U128;
        aborts_if b != c && a <= c && c == 0;
        //a * b overflow
        aborts_if b != c && a <= c && a / c * (a % c) > MAX_U128;
        //a * b * c overflow
        aborts_if b != c && a <= c && a / c * (a % c) * c > MAX_U128;
        //a * d overflow
        aborts_if b != c && a <= c && a / c * (b % c) > MAX_U128;
        //a * b * c + a * d overflow
        aborts_if b != c && a <= c && a / c * (a % c) * c + a / c * (b % c) > MAX_U128;
        //b * c overflow
        aborts_if b != c && a <= c && a % c * (b / c) > MAX_U128;
        //b * d overflow
        aborts_if b != c && a <= c && a % c * (b % c) > MAX_U128;
        //b * d / c overflow
        aborts_if b != c && a <= c && a % c * (b % c) / c > MAX_U128;
        //a * b * c + a * d + b * c overflow
        aborts_if b != c && a <= c && a / c * (a % c) * c + a / c * (b % c) + a % c * (b / c) > MAX_U128;
        //a * b * c + a * d + b * c + b * d / c overflow
        aborts_if b != c && a <= c && a / c * (a % c) * c + a / c * (b % c) + a % c * (b / c) + a % c * (b % c) / c > MAX_U128;
    }

    spec fun spec_mul_div(): u128;
}
