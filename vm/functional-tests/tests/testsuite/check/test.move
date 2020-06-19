//! account: alice, 90000STC
//! account: bob, 90000STC

//! new-transaction

//! sender: alice

module M {
    use 0x1::Signer;

    resource struct Cup {
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

    public fun t5(account: &signer) {
        move_to(account, Cup { a: fail(0) });
    }

    public fun destroy(sender: &signer): u64 acquires Cup {
        let Cup { a } = move_from<Cup>(Signer::address_of(sender));
        a
    }

    fun fail(code: u64): u64 {
        abort code
    }
}


//! new-transaction
//! sender: bob

script {
use {{alice}}::M;
fun main(account: &signer) {
  let cup = M::new();
  M::publish(cup, account)
}
}

// check: EXECUTED
// check: delta_size
// check: 8


//! new-transaction
//! sender: bob

script {
use {{alice}}::M;
fun main(account: &signer) {
  let cup = M::new();
  M::publish(cup, account);
  let y = M::destroy(account);
  assert(y == 1, 41)
}
}

// check: EXECUTED
// check: delta_size
// check: 0
