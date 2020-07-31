//! account: alice, 9000000 0x1::STC::STC
//! account: bob

//! sender: alice
module Example {
    public fun new(): R {
        R { x: true }
    }

    public fun destroy(r: R) {
        R { x: _ } = r;
    }

    resource struct R { x: bool }

    public fun save(account: &signer, r: R){
        move_to(account, r);
    }

    public fun get_x(account: &signer): bool acquires R {
        let sender = Signer::address_of(account);
        assert(exists<R>(sender), 1);
        let r = borrow_global<R>(sender);
        r.x
    }

    use 0x1::Signer;
}

//! new-transaction
//! sender: alice
script {
use {{alice}}::Example;
fun main() {
    let r = Example::new();
    Example::destroy(r);
}
}

//! new-transaction
//! sender: bob
script {
use {{alice}}::Example;
fun main(account: &signer) {
    let r = Example::new();
    Example::save(account, r);
    assert(Example::get_x(account), 1);
}
}