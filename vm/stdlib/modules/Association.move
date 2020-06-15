// Implements logic for registering addresses as association addresses, and
// determining if the sending account is an association account.
// Errors:
// 1000 -> INVALID_GENESIS_ADDR
// 1001 -> INSUFFICIENT_PRIVILEGES
// 1002 -> NOT_AN_ASSOCIATION_ACCOUNT
// 1003 -> ACCOUNT_DOES_NOT_HAVE_PRIVILEGE
// 1004 -> ACCOUNT_DOES_NOT_HAVE_PRIVILEGE_RESOURCE
address 0x1 {

module Association {

    use 0x1::Signer;

    // The root account privilege. This is created at genesis and has
    // special privileges (e.g. removing an account as an association
    // account). It cannot be removed.
    resource struct Root { }

    // There are certain association capabilities that are more
    // privileged than other association operations. This resource with the
    // type representing that privilege is published under the privileged
    // account.
    resource struct PrivilegedCapability<Privilege> {}

    // A type tag to mark that this account is an association account.
    // It cannot be used for more specific/privileged operations.
    struct T { }

    // Initialization is called in genesis. It publishes the root resource
    // under the root_address() address, marks it as a normal
    // association account.
    public fun initialize(association: &signer) {
        assert(Signer::address_of(association) == root_address(), 1000);
        move_to(association, Root{ });
        move_to(association, PrivilegedCapability<T>{ });
    }

   /// Certify the privileged capability published under `association`.
   public fun grant_privilege<Privilege>(association: &signer, recipient: &signer) {
         assert_is_root(association);
         move_to(recipient, PrivilegedCapability<Privilege>{ });
   }

   /// Grant the association privilege to `association`
       public fun grant_association_address(association: &signer, recipient: &signer) {
           grant_privilege<T>(association, recipient)
       }

       /// Return whether the `addr` has the specified `Privilege`.
       public fun has_privilege<Privilege>(addr: address): bool {
           // TODO: make genesis work with this check enabled
           //addr_is_association(addr) &&
           exists<PrivilegedCapability<Privilege>>(addr)
       }

       /// Remove the `Privilege` from the address at `addr`. The `sender` must be the root association
       /// account.
       /// Aborts if `addr` is the address of the root account
       public fun remove_privilege<Privilege>(association: &signer, addr: address)
       acquires PrivilegedCapability {
           assert_is_root(association);
           // root should not be able to remove its own privileges
           assert(Signer::address_of(association) != addr, 1005);
           assert(exists<PrivilegedCapability<Privilege>>(addr), 1004);
           PrivilegedCapability<Privilege>{ } = move_from<PrivilegedCapability<Privilege>>(addr);
       }

       /// Assert that the sender is an association account.
       public fun assert_is_association(account: &signer) {
           assert_addr_is_association(Signer::address_of(account))
       }

       /// Assert that the sender is the root association account.
       public fun assert_is_root(account: &signer) {
           assert(exists<Root>(Signer::address_of(account)), 1001);
       }

       /// Return whether the account at `addr` is an association account.
       public fun addr_is_association(addr: address): bool {
           exists<PrivilegedCapability<T>>(addr)
       }

       /// The address at which the root account will be published.
       public fun root_address(): address {
           0xA550C18
       }

       /// Assert that `addr` is an association account.
       fun assert_addr_is_association(addr: address) {
           assert(addr_is_association(addr), 1002);
       }
}

}
