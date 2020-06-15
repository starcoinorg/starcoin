//! account: alice, 90000STC
//! account: bob, 90000STC

//! sender: alice

module A {
    resource struct Coin {
        u: u64,
        n: u64,
        w: u64,
    }

    public fun new(): Coin {
        Coin { u: 1, n: 1, w: 1 }
    }
}

//! new-transaction
//! sender: bob

module Tester {
    use {{alice}}::A;
    use 0x0::Signer;
    use 0x0::Transaction;

    resource struct Pair { x: A::Coin, y: A::Coin }

    public fun test_eq(addr1: address, addr2: address): bool acquires Pair {
        let p1 = borrow_global<Pair>(addr1);
        let p2 = borrow_global<Pair>(addr2);
        p1 == p2
    }

    public fun test(account: &signer) acquires Pair {
        move_to<Pair>(account, Pair { x: A::new(), y: A::new() });
        Transaction::assert(test_eq(Signer::address_of(account), Signer::address_of(account)), 42);
    }

}

//! new-transaction
//! sender: bob
script {
use {{bob}}::Tester;

fun main(account: &signer) {
    Tester::test(account);
}
}

// check: EXECUTED
// check: increment_size
// check: 1