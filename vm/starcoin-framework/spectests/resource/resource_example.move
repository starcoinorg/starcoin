//# init -n dev

//# faucet --addr alice --amount 9000000

//# faucet --addr bob


//# publish
module alice::Example {
    use StarcoinFramework::Signer;
    public fun new(): R {
        R { x: true }
    }

    public fun destroy(r: R) {
        R { x: _ } = r;
    }

    struct R has key, store { x: bool }

    public fun save(account: &signer, r: R){
        move_to(account, r);
    }

    public fun get_x(account: &signer): bool acquires R {
        let sender = Signer::address_of(account);
        assert!(exists<R>(sender), 1);
        let r = borrow_global<R>(sender);
        r.x
    }
}

//# run --signers alice
script {
use alice::Example;
fun main() {
    let r = Example::new();
    Example::destroy(r);
}
}

//# run --signers bob
script {
use alice::Example;
fun main(account: signer) {
    let r = Example::new();
    Example::save(&account, r);
    assert!(Example::get_x(&account), 1);
}
}