//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//#publish
module alice::N {
    public fun bar() {}
}

//#publish
module bob::M {
    use alice::N;
    public fun foo() {
        N::bar()
    }
}

// check: EXECUTED

//# publish
module alice::N {
    use bob::M;
    public fun bar() {
      M::foo()
    }
}

// check: "ERROR { status_code: \"CYCLIC_MODULE_DEPENDENCY\" }"