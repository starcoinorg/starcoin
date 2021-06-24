//! account: alice, 90000 0x1::STC::STC
//! account: bob, 90000 0x1::STC::STC

//! sender: alice
address alice = {{alice}};
address bob = {{bob}};
module alice::N {
    public fun bar() {}
}

//! new-transaction
//! sender: bob
address alice = {{alice}};
address bob = {{bob}};
module bob::M {
    use alice::N;
    public fun foo() {
        N::bar()
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
address alice = {{alice}};
address bob = {{bob}};
module alice::N {
    use bob::M;
    public fun bar() {
      M::foo()
    }
}

// check: "ERROR { status_code: CYCLIC_MODULE_DEPENDENCY }"