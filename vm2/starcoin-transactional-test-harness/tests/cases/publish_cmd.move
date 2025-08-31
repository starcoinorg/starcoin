//# init -n dev

//# faucet --addr creator --amount 100000000000

//# publish
module creator::test {
    struct Foo has key {
        x: u64,
    }

    public fun publish_foo(s: &signer) {
        move_to<Foo>(s, Foo { x: 500 })
    }
}

//# run --signers creator
script {
    use creator::test;

    fun main(s: signer) {
        test::publish_foo(&s);
    }
}


