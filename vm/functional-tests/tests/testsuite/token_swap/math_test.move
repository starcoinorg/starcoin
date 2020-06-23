//! account: alice
//! account: bob

//! sender: alice
module Math {
    // babylonian method (https://en.wikipedia.org/wiki/Methods_of_computing_square_roots#Babylonian_method)
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
}
// check: EXECUTED
// check: 0

//! new-transaction
//! sender: bob
script {
    use {{alice}}::Math::sqrt;
    fun main(_signer: &signer) {
        assert(sqrt(0) == 0, 0);
        assert(sqrt(1) == 1, 1);
        assert(sqrt(2) == 1, 1);
        assert(sqrt(3) == 1, 1);

        assert(sqrt(4) == 2, 2);
        assert(sqrt(5) == 2, 2);

        assert(sqrt(9) == 3, 3);
        assert(sqrt(15) == 3, 3);
        assert(sqrt(16) == 4, 5);
    }
}
// check: EXECUTED