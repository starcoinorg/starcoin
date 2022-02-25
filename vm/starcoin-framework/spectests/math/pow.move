//# init -n dev

//# faucet --addr creator

//# run --signers creator
script {
    use StarcoinFramework::Math::pow;
    fun main(_signer: signer) {
        assert!(pow(1, 2) == 1, 0);
        assert!(pow(2, 1) == 2, 1);
        assert!(pow(2, 2) == 4, 1);
        assert!(pow(3, 4) == 81, 1);
    }
}

// check: EXECUTED

//# run --signers creator
script {
    use StarcoinFramework::Math::pow;
    fun main()  {
        // test overflow
        assert!(pow(18446744073709551614, 2) == 0, 3);
    }
}

// check: ARITHMETIC_ERROR



