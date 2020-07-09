address 0x1 {
module CoreAddresses {

    /// The address of the genesis
    public fun GENESIS_ACCOUNT(): address {
        0x1
    }

    /// The address of the root association account. This account is
    /// created in genesis, and cannot be changed. This address has
    /// ultimate authority over the permissions granted (or removed) from
    /// accounts on-chain.
    public fun ASSOCIATION_ROOT_ADDRESS(): address {
        0xA550C18
    }

    /// The reserved address for transactions inserted by the VM into blocks (e.g.
    /// block metadata transactions). Because the transaction is sent from
    /// the VM, an account _cannot_ exist at the `0x0` address since there
    /// is no signer for the transaction.
    public fun VM_RESERVED_ADDRESS(): address {
        0x0
    }
}
}
