//# init -n dev

//# faucet --addr creator --amount 100000000000

//# publish
module creator::test {
     struct TestObj has store{
        addr: address,
    }

    fun use_field(_addr: address){}

    fun borrow_field(obj: &TestObj){
        use_field(obj.addr)
    }


    public fun test_borrow_field(addr: address) {
        let obj = TestObj{addr};
        borrow_field(&obj);
        let TestObj{addr:_} = obj;
    }
}

//# run --signers creator
script {
    use StarcoinFramework::Signer;
    use creator::test;

    fun main(s: signer) {
        let addr = signer::address_of(&s);
        test::test_borrow_field(addr);
    }
}


