//! max-gas: 2000000
script {
    use 0x1::Vector;
    use 0x1::Math::{sum,avg};
    fun main(_signer: signer) {
        let nums = Vector::empty();
        let i = 1;
        while (i <= 100) {
            Vector::push_back(&mut nums, (i as u128));
            i = i + 1;
        };
        assert(sum(&nums) == 5050, 1000);
        assert(avg(&nums) == 50, 1001);
    }
}



