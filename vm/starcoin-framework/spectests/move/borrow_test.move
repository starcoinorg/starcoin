//# init -n dev

//# faucet --addr alice --amount 90000

//# faucet --addr bob --amount 90000

//# publish
module alice::M {
    use StarcoinFramework::Signer;

    struct M has key, store {
        value: u64,
    }

    public fun new(): M {
         M { value: 1 }
    }

    public fun value(this: &M) : u64 {
        this.value
    }

    public fun save(account: &signer, m: M){
        move_to(account, m);
    }

    public fun get(account: &signer):M acquires M {
        move_from<M>(Signer::address_of(account))
    }

    public fun get_value(account: &signer): u64 acquires M {
        let m = borrow_global<M>(Signer::address_of(account));
        m.value
    }

    public fun destroy(m: M): u64 {
        let M { value } = m;
        value
    }
}

//# run --signers alice
script {
use alice::M;

fun main(account: signer) {
    let m = M::new();
    M::save(&account, m);
}
}

// check: EXECUTED

//# run --signers alice
script {
use alice::M;

fun main(account: signer) {
    let v = M::get_value(&account);
    assert!(v == 1, 80001);
}
}

// check: EXECUTED

//# run --signers alice
script {
use alice::M;

fun main(account: signer) {
    let m = M::get(&account);
    let v = M::destroy(m);
    assert!(v == 1, 80001);
}
}

// check: EXECUTED