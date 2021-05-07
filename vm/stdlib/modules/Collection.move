address 0x1 {
/// Deprecated since @v3 please use Collection2
/// Provide a account based vector for save resource.
module Collection {
    use 0x1::Signer;
    use 0x1::Errors;
    use 0x1::Vector;
    use 0x1::Option::{Self, Option};

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = false;
    }

    /// Collection in memory, can not drop & store.
    struct Collection<T>{
        items:vector<T>,
        /// the owner of Collection.
        owner: address,
    }

    /// Collection in global store.
    struct CollectionStore<T: store> has key {
        /// items in the CollectionStore.
        /// use Option at  here is for temporary take away from store to construct Collection.
        items: Option<vector<T>>,
    }

    const EDEPRECATED_FUNCTION: u64 = 11;

    const ECOLLECTION_NOT_EXIST: u64 = 101;
    /// The operator require the collection owner.
    const ECOLLECTION_NOT_OWNER: u64 = 102;

    /// Return the length of the collection.
    public fun length<T>(c: &Collection<T>): u64{
        Vector::length(&c.items)
    }

    spec fun length {aborts_if false;}

    /// Acquire an immutable reference to the `i`th element of the collection `c`.
    /// Aborts if `i` is out of bounds.
    public fun borrow<T>(c: &Collection<T>, i: u64): &T{
        Vector::borrow(&c.items, i)
    }

    /// Deprecated since @v3
    /// Add item `v` to the end of the collection `c`.
    /// require owner of Collection.
    public fun push_back<T>(_account: &signer, _c: &mut Collection<T>, _t: T){
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    /// Return a mutable reference to the `i`th item in the Collection `c`.
    /// Aborts if `i` is out of bounds.
    public fun borrow_mut<T>(account: &signer, c: &mut Collection<T>, i: u64): &mut T{
        assert(Signer::address_of(account) == c.owner, Errors::requires_address(ECOLLECTION_NOT_OWNER));
        Vector::borrow_mut<T>(&mut c.items, i)
    }

    /// Pop an element from the end of vector `v`.
    /// Aborts if `v` is empty.
    public fun pop_back<T>(account: &signer, c: &mut Collection<T>): T {
        assert(Signer::address_of(account) == c.owner, Errors::requires_address(ECOLLECTION_NOT_OWNER));
        Vector::pop_back<T>(&mut c.items)
    }

    public fun remove<T>(account: &signer, c: &mut Collection<T>, i: u64): T{
        assert(Signer::address_of(account) == c.owner, Errors::requires_address(ECOLLECTION_NOT_OWNER));
        Vector::remove<T>(&mut c.items, i)
    }

    /// Deprecated since @v3
    public fun append<T>(_account: &signer, _c: &mut Collection<T>, _other: T) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    /// Deprecated since @v3
    public fun append_all<T>(_account: &signer, _c: &mut Collection<T>, _other: vector<T>) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    /// Check if the addr has T
    public fun has<T: store>(addr: address): bool {
        // just return exists CollectionStore<T>, because we will ensure at least one item in CollectionStore.
        exists_at<T>(addr)
    }

    /// check the Collection exists in `addr`
    fun exists_at<T: store>(addr: address): bool{
        exists<CollectionStore<T>>(addr)
    }

    spec fun exists_at {aborts_if false;}


    /// Deprecated since @v3
    /// Put items to account's Collection last position.
    public fun put<T: store>(_account: &signer, _item: T) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    spec fun put {aborts_if false;}

    /// Deprecated since @v3
    /// Put itemss to account's box last position.
    public fun put_all<T: store>(_account: &signer, _items: vector<T>) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    spec fun put_all {aborts_if false;}

    /// Take last item from account's Collection of T.
    public fun take<T: store>(account: &signer): T acquires CollectionStore{
        let addr = Signer::address_of(account);
        let c = borrow_collection<T>(addr);
        let item = pop_back(account, &mut c);
        return_collection(c);
        item
    }

    spec fun take {
        aborts_if false;
    }

    /// Borrow collection of T from `addr`
    public fun borrow_collection<T: store>(addr: address): Collection<T> acquires CollectionStore{
        assert(exists_at<T>(addr), Errors::invalid_state(ECOLLECTION_NOT_EXIST));
        let c = borrow_global_mut<CollectionStore<T>>(addr);
        let items = Option::extract(&mut c.items);
        Collection{
            items,
            owner: addr
        }
    }

    spec fun borrow_collection {
        aborts_if false;
    }

    /// Return the Collection of T
    public fun return_collection<T: store>(c: Collection<T>) acquires CollectionStore{
        let Collection{ items, owner } = c;
        if (Vector::is_empty(&items)) {
            let c = move_from<CollectionStore<T>>(owner);
            destroy_empty(c);
            Vector::destroy_empty(items);
        }else{
            let c = borrow_global_mut<CollectionStore<T>>(owner);
            Option::fill(&mut c.items, items);
        }
    }

    spec fun return_collection {
        aborts_if false;
    }

    fun destroy_empty<T: store>(c: CollectionStore<T>){
        let CollectionStore{ items } = c;
        Option::destroy_none(items);
    }

    spec fun destroy_empty {
        aborts_if false;
    }

}
}