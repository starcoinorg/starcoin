//# init -n dev

//# faucet --addr creator --amount 100000000000

//# publish
module creator::test {
    public fun hello(){}   
}

//# run --signers creator
script{
    use creator::test;
    fun main(_sender: signer){
        test::hello();
    }
}