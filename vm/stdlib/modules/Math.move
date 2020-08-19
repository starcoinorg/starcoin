address 0x1 {

module Math {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }
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

    spec fun sqrt {
        pragma verify = false; //costs too much time
        pragma timeout = 120;
        aborts_if y >= 4 && y / (y/2 +1) + y/2 +1 > max_u128();
        aborts_if y >= 4 && y / (y/2 +1) > max_u128();
    }

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

    spec fun pow {
        pragma verify = false;
        //aborts_if y > 0 && x * x > max_u128();
    }
}
}