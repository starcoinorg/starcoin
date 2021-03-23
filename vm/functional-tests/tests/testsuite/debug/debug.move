//! debug

module M {
    use 0x1::Debug;
    use 0x1::Vector;

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
        Debug::print<u64>(&x);

        v = Vector::empty<u64>();
        Vector::push_back<u64>(&mut v, 100);
        Vector::push_back<u64>(&mut v, 200);
        Vector::push_back<u64>(&mut v, 300);
        Debug::print<vector<u64>>(&v);

        foo = Foo { x: false };
        Debug::print<Foo>(&foo);

        bar = Bar { x: 404u128, y: Foo { x: false }, z: true };
        Debug::print<Bar>(&bar);

        box = Collection<Foo> { x: Foo { x: false } };
        Debug::print<Collection<Foo>>(&box);
    }
}
// check: EXECUTED

//! new-transaction
script {
use {{default}}::M;

fun main() {
    M::test();
 }
}

// check: EXECUTED
