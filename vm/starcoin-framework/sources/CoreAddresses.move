address StarcoinFramework {
/// The module provide addresses used in stdlib.    
module CoreAddresses {
    use StarcoinFramework::Signer;
    use StarcoinFramework::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    const ENOT_GENESIS_ACCOUNT: u64 = 11;

    /// The address of the genesis
    public fun GENESIS_ADDRESS(): address {
        @0x1
    }

    /// Specification version of `Self::GENESIS_ACCOUNT`.

    spec fun SPEC_GENESIS_ADDRESS(): address {
        @0x1
    }


    /// Assert signer is genesis.
    public fun assert_genesis_address(account: &signer) {
        assert!(Signer::address_of(account) == GENESIS_ADDRESS(), Errors::requires_address(ENOT_GENESIS_ACCOUNT))
    }
    spec assert_genesis_address {
        pragma opaque;
        include AbortsIfNotGenesisAddress;
    }

    /// Specifies that a function aborts if the account does not have the Diem root address.
    spec schema AbortsIfNotGenesisAddress {
        account: signer;
        aborts_if Signer::address_of(account) != SPEC_GENESIS_ADDRESS();
    }

    /// The address of the root association account. This account is
    /// created in genesis, and cannot be changed. This address has
    /// ultimate authority over the permissions granted (or removed) from
    /// accounts on-chain.
    public fun ASSOCIATION_ROOT_ADDRESS(): address {
        @0xA550C18
    }

    /// Specification version of `Self::ASSOCIATION_ROOT_ADDRESS`.

    spec fun SPEC_ASSOCIATION_ROOT_ADDRESS(): address {
        @0xA550C18
    }


    /// The reserved address for transactions inserted by the VM into blocks (e.g.
    /// block metadata transactions). Because the transaction is sent from
    /// the VM, an account _cannot_ exist at the `0x0` address since there
    /// is no signer for the transaction.
    public fun VM_RESERVED_ADDRESS(): address {
        @0x0
    }

    /// Specification version of `Self::VM_RESERVED_ADDRESS`.

    spec fun SPEC_VM_RESERVED_ADDRESS(): address {
        @0x0
    }

}
}
