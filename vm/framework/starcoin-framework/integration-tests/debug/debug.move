//# init -n dev

//# faucet --addr default --amount 100000000000000000

//# publish
module default::M {
    use starcoin_framework::debug;
    use starcoin_framework::vector;

    struct Foo has copy, drop, store { x: bool }

    struct Bar has copy, drop, store { x: u128, y: Foo, z: bool }

    struct Collection<T> has copy, drop, store { x: T }

    public fun test() {
        let x: u64;
        let v: vector<u64>;
        let foo: Foo;
        let bar: Bar;
        let box: Collection<Foo>;

        x = 42;
        debug::print<u64>(&x);

        v = vector::empty<u64>();
        vector::push_back<u64>(&mut v, 100);
        vector::push_back<u64>(&mut v, 200);
        vector::push_back<u64>(&mut v, 300);
        debug::print<vector<u64>>(&v);

        foo = Foo { x: false };
        debug::print<Foo>(&foo);

        bar = Bar { x: 404u128, y: Foo { x: false }, z: true };
        debug::print<Bar>(&bar);

        box = Collection<Foo> { x: Foo { x: false } };
        debug::print<Collection<Foo>>(&box);
    }
}
// check: EXECUTED

//# run --signers default
script {
    use default::M;

    fun main() {
        M::test();
    }
}

// check: EXECUTED
