//# init -n dev

//# faucet --addr creator --amount 100000000000

//# package
module creator::test {
    public fun hello(){}   
}

//# deploy {{$.package[0].file}}

//# run --signers creator
script{
    use creator::test;
    fun main(_sender: signer){
        test::hello();
    }
}