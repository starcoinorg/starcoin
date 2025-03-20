address StarcoinFramework {
/// Provide access methods for Signer.
module Signer {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }
    /// Borrows the address of the signer
    /// Conceptually, you can think of the `signer` as being a resource struct wrapper around an
    /// address
    /// ```
    /// resource struct Signer has key, store { addr: address }
    /// ```
    /// `borrow_address` borrows this inner field
    native public fun borrow_address(s: &signer): &address;

    /// Copies the address of the signer
    public fun address_of(s: &signer): address {
        *borrow_address(s)
    }

    spec address_of {
        pragma opaque = true;
        aborts_if false;
        ensures result == address_of(s);
    }
}
}
