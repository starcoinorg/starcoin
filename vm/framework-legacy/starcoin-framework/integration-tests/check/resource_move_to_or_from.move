//# init -n dev

//# faucet --addr alice --amount 900000

//# faucet --addr bob --amount 900000



//# publish

module alice::M {
    use StarcoinFramework::Signer;

    struct Cup has key, store {
        a: u64,
    }
    public fun new(): Cup {
      Cup { a: 1 }
    }
    public fun get_a(this: &Cup): u64 {
        this.a
    }

    public fun publish(cup: Cup, account: &signer) {
      move_to(account, cup);
    }

    public fun get_cup(account: &signer): Cup acquires Cup {
        let cup = borrow_global<Cup>(Signer::address_of(account));
        Cup { a: cup.a }
    }

    public fun destroy(sender: &signer): u64 acquires Cup {
        let Cup { a } = move_from<Cup>(Signer::address_of(sender));
        a
    }

    public fun destroy_x(x: Cup) {
        Cup { a: _ } = x;
    }
}



//# run --signers bob

script {
use alice::M;
fun main(account: signer) {
  let cup = M::new();
  M::publish(cup, &account);
}
}

// check: EXECUTED
//// check: delta_size 8



//# run --signers bob

script {
use alice::M;
fun main(account: signer) {
  let y = M::destroy(&account);
  assert!(y == 1, 41)
}
}

// check: EXECUTED
//// check: delta_size -8



//# run --signers bob

script {
use alice::M;
fun main(account: signer) {
    let cup = M::new();
    M::publish(cup, &account);
    let y = M::destroy(&account);
    assert!(y == 1, 41);
}
}

// check: EXECUTED
//// check: delta_size 0


//# run --signers bob

script {
use alice::M;
fun main(account: signer) {
    let cup = M::new();
    M::publish(cup, &account);
    let cup = M::get_cup(&account);
    M::destroy_x(cup)
}
}

// check: EXECUTED
//// check: delta_size 8

