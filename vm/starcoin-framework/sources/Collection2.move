address StarcoinFramework {
/// Provide a account based vector for save resource item.
/// The resource in CollectionStore can borrowed by anyone, anyone can get immutable ref of item.
/// and the owner of Collection can allow others to add item to Collection or get mut ref from Collection.git
module Collection2 {
    use StarcoinFramework::Signer;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Vector;
    use StarcoinFramework::Option::{Self, Option};

    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict = false;
    }

    /// Collection in memory, can not drop & store.
    struct Collection<T>{
        items: vector<T>,
        owner: address,
        can_put: bool,
        can_mut: bool,
        can_take: bool,
    }

    /// Collection in global store.
    struct CollectionStore<T: store> has key {
        /// items in the CollectionStore.
        /// use Option at here is for temporary take away from store to construct Collection.
        items: Option<vector<T>>,
        anyone_can_put: bool,
        anyone_can_mut: bool,
    }

    const ERR_COLLECTION_NOT_EXIST: u64 = 101;
    /// The operator require the collection owner or collection set anyone_can_put to true.
    const ERR_COLLECTION_CAN_NOT_ADD: u64 = 102;
    /// The operator require the collection owner or collection set anyone_can_mut to true.
    const ERR_COLLECTION_CAN_NOT_MUT: u64 = 103;
   /// The operator require the collection owner
    const ERR_COLLECTION_CAN_NOT_TAKE: u64 = 104;
    const ERR_COLLECTION_INVALID_BORROW_STATE: u64 = 105;
    const ERR_COLLECTION_IS_NOT_EMPTY: u64 = 106;

    /// Return the length of the collection.
    public fun length<T>(c: &Collection<T>): u64{
        Vector::length(&c.items)
    }

    spec length {aborts_if false;}

    /// Acquire an immutable reference to the `i`th element of the collection `c`.
    /// Aborts if `i` is out of bounds.
    public fun borrow<T>(c: &Collection<T>, i: u64): &T{
        Vector::borrow(&c.items, i)
    }

    /// Add item `v` to the end of the collection `c`.
    /// require owner of Collection.
    public fun push_back<T>(c: &mut Collection<T>, t: T){
        assert!(c.can_put, Errors::requires_address(ERR_COLLECTION_CAN_NOT_ADD));
        Vector::push_back<T>(&mut c.items, t);
    }

    /// Return a mutable reference to the `i`th item in the Collection `c`.
    /// Aborts if `i` is out of bounds.
    public fun borrow_mut<T>(c: &mut Collection<T>, i: u64): &mut T{
        assert!(c.can_mut, Errors::requires_address(ERR_COLLECTION_CAN_NOT_MUT));
        Vector::borrow_mut<T>(&mut c.items, i)
    }

    /// Pop an element from the end of vector `v`.
    /// Aborts if `v` is empty.
    public fun pop_back<T>(c: &mut Collection<T>): T {
        assert!(c.can_take, Errors::requires_address(ERR_COLLECTION_CAN_NOT_TAKE));
        Vector::pop_back<T>(&mut c.items)
    }

    public fun remove<T>(c: &mut Collection<T>, i: u64): T{
        assert!(c.can_take, Errors::requires_address(ERR_COLLECTION_CAN_NOT_TAKE));
        Vector::remove<T>(&mut c.items, i)
    }

    public fun append<T>(c: &mut Collection<T>, other: T) {
        assert!(c.can_put, Errors::requires_address(ERR_COLLECTION_CAN_NOT_ADD));
        Vector::append<T>(&mut c.items, Vector::singleton(other))
    }

    public fun append_all<T>(c: &mut Collection<T>, other: vector<T>) {
        assert!(c.can_put, Errors::requires_address(ERR_COLLECTION_CAN_NOT_ADD));
        Vector::append<T>(&mut c.items, other)
    }

    /// check the Collection exists in `addr`
    public fun exists_at<T: store>(addr: address): bool{
        exists<CollectionStore<T>>(addr)
    }

    spec exists_at {aborts_if false;}

    /// check `addr` is accept T and other can send T to `addr`,
    /// it means exists a Collection of T at `addr` and anyone_can_put of the Collection is true
    public fun is_accept<T: store>(addr: address): bool acquires CollectionStore {
        if (!exists<CollectionStore<T>>(addr)){
            return false
        };
        let cs = borrow_global<CollectionStore<T>>(addr);
        cs.anyone_can_put
    }

    /// signer allow other send T to self
    /// create a Collection of T and set anyone_can_put to true
    /// if the Collection exists, just update anyone_can_put to true
    public fun accept<T: store>(signer: &signer) acquires CollectionStore {
         let addr = Signer::address_of(signer);
        if (!exists<CollectionStore<T>>(addr)){
            Self::create_collection<T>(signer, true, false);
        };
        let cs = borrow_global_mut<CollectionStore<T>>(addr);
        if (!cs.anyone_can_put) {
            cs.anyone_can_put = true;
        }
    }

    /// Put items to `to_addr`'s Collection of T
    /// put = borrow_collection<T> + append + return_collection.
    public fun put<T: store>(signer: &signer, owner: address, item: T) acquires CollectionStore{
        let c = Self::borrow_collection(signer, owner);
        Self::append(&mut c, item);
        Self::return_collection(c);
    }

    spec put {aborts_if false;}

    /// Put all items to owner's collection of T.
    public fun put_all<T: store>(signer: &signer, owner: address, items: vector<T>) acquires CollectionStore{
        let c = Self::borrow_collection(signer, owner);
        Self::append_all(&mut c, items);
        Self::return_collection(c);
    }

    spec put_all {aborts_if false;}

    /// Take last item from signer's Collection of T.
    public fun take<T: store>(signer: &signer): T acquires CollectionStore{
        let addr = Signer::address_of(signer);
        let c = borrow_collection<T>(signer, addr);
        let item = pop_back(&mut c);
        return_collection(c);
        item
    }

    spec take {
        aborts_if false;
    }

    public fun create_collection<T:store>(signer: &signer, anyone_can_put: bool, anyone_can_mut: bool) {
        move_to(signer, CollectionStore<T>{items: Option::some(Vector::empty<T>()), anyone_can_put, anyone_can_mut})
    }

    /// Return the length of Collection<T> from `owner`, if collection do not exist, return 0.
    public fun length_of<T: store>(owner: address) : u64 acquires CollectionStore{
        if (exists_at<T>(owner)){
            let cs = borrow_global_mut<CollectionStore<T>>(owner);
            //if items is None, indicate it is borrowed
            assert!(!Option::is_none(&cs.items), Errors::invalid_state(ERR_COLLECTION_INVALID_BORROW_STATE));
            let items = Option::borrow(&cs.items);
            Vector::length(items)
        }else{
            0
        }
    }

    /// Borrow collection of T from `owner`, auto detected the collection's can_put|can_mut|can_take by the `sender` and Collection config.
    public fun borrow_collection<T: store>(sender: &signer, owner: address): Collection<T> acquires CollectionStore{
        assert!(exists_at<T>(owner), Errors::invalid_state(ERR_COLLECTION_NOT_EXIST));
        let cs = borrow_global_mut<CollectionStore<T>>(owner);
        //if items is None, indicate it is borrowed
        assert!(!Option::is_none(&cs.items), Errors::invalid_state(ERR_COLLECTION_INVALID_BORROW_STATE));
        let items = Option::extract(&mut cs.items);
        let is_owner = owner == Signer::address_of(sender);
        let can_put = cs.anyone_can_put || is_owner;
        let can_mut = cs.anyone_can_mut || is_owner;
        let can_take = is_owner;
        Collection{
            items,
            owner,
            can_put,
            can_mut,
            can_take,
        }
    }

    spec borrow_collection {
        aborts_if false;
    }

    /// Return the Collection of T
    public fun return_collection<T: store>(c: Collection<T>) acquires CollectionStore{
        let Collection{ items, owner, can_put:_, can_mut:_, can_take:_ } = c;
        let cs = borrow_global_mut<CollectionStore<T>>(owner);
        assert!(Option::is_none(&cs.items), Errors::invalid_state(ERR_COLLECTION_INVALID_BORROW_STATE));
        Option::fill(&mut cs.items, items);
    }

    spec return_collection {
        aborts_if false;
    }

    public fun destroy_collection<T: store>(signer: &signer) acquires CollectionStore{
        let c = move_from<CollectionStore<T>>(Signer::address_of(signer));
        destroy_empty(c);
    }

    spec destroy_collection {
        aborts_if false;
    }

    fun destroy_empty<T: store>(c: CollectionStore<T>){
        let CollectionStore{ items, anyone_can_put:_, anyone_can_mut:_,} = c;
        if (Option::is_some(&items)) {
            let item_vec = Option::extract(&mut items);
            assert!(Vector::is_empty(&item_vec), Errors::invalid_state(ERR_COLLECTION_IS_NOT_EMPTY));
            Vector::destroy_empty(item_vec);
            Option::destroy_none(items);
        }else{
            Option::destroy_none(items);
        }
    }

    spec destroy_empty {
        aborts_if false;
    }

}
}