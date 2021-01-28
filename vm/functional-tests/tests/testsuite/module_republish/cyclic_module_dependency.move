//! account: alice, 90000 0x1::STC::STC
//! account: bob, 90000 0x1::STC::STC

//! sender: alice

module N {
    public fun bar() {}
}

//! new-transaction
//! sender: bob

module M {
    use {{alice}}::N;
    public fun foo() {
        N::bar()
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice

module N {
    use {{bob}}::M;
    public fun bar() {
      M::foo()
    }
}

// check: "ERROR { status_code: CYCLIC_MODULE_DEPENDENCY }"