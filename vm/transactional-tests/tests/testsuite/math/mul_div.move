//# init -n dev

//# faucet --addr creator

//# run --signers creator
script {
    use StarcoinFramework::Math::{Self,mul_div};
    fun main(_signer: signer) {
        assert!(mul_div(1, 1, 2) == 0, 1000);
        assert!(mul_div(2, 1, 2) == 1, 1001);
        assert!(mul_div(2, 1, 1) == 2, 1002);
        assert!(mul_div(100, 1, 2) == 50, 1003);

        assert!(mul_div((Math::u64_max() as u128), 1, 2) ==  (Math::u64_max()/2 as u128), 1004);
        assert!(mul_div((Math::u64_max() as u128), (Math::u64_max()/2 as u128), (Math::u64_max() as u128)) == (Math::u64_max()/2 as u128), 1005);
        assert!(mul_div(Math::u128_max(), 1, 2) ==  Math::u128_max()/2, 1006);
        assert!(mul_div(Math::u128_max(), Math::u128_max()/2, Math::u128_max()) ==  Math::u128_max()/2, 1007);

        assert!(mul_div(100, 1, 3) == 33, 1008);
        assert!(mul_div(100, 1000, 3000) == 33, 1009);
        assert!(mul_div(100, 2, 101) == 1, 1010);
        assert!(mul_div(100, 50, 101) == 49, 1011);
        assert!(mul_div(100, 1000, 101) == 990, 1012);
        assert!(mul_div(100, 1000, 1) == 100000, 1013);
        assert!(mul_div(1, 100, 1) == 100, 1014);
        assert!(mul_div(500000000000000u128, 899999999999u128, 1399999999999u128) == 321428571428443, 1015);

    }
}



