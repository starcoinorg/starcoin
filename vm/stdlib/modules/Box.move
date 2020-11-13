address 0x1 {
// Provider a account based vector for save resource.
module Box {
    use 0x1::Signer;
    use 0x1::Errors;
    use 0x1::Vector;

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    resource struct Box<T>{
        thing:vector<T>,
    }

    const EBOX_NOT_EXIST: u64 = 101;

    public fun exists_at<T>(addr: address): bool{
        exists<Box<T>>(addr)
    }

    spec fun exists_at {aborts_if false;}

    public fun length<T>(addr: address): u64 acquires Box{
        if (exists_at<T>(addr)) {
            let box = borrow_global<Box<T>>(addr);
            Vector::length(&box.thing)
        }else{
           0
        }
    }

    spec fun length {aborts_if false;}

    // Put thing to account's box last position.
    public fun put<T>(account: &signer, thing: T) acquires Box{
        let addr = Signer::address_of(account);
        if (exists_at<T>(addr)) {
            let box = borrow_global_mut<Box<T>>(addr);
            Vector::push_back(&mut box.thing, thing);
        }else{
            move_to(account, Box<T>{thing: Vector::singleton(thing)})
        }
    }

    spec fun put {aborts_if false;}

    public fun put_all<T>(account: &signer, thing: vector<T>) acquires Box{
        let addr = Signer::address_of(account);
        if (exists_at<T>(addr)) {
            let box = borrow_global_mut<Box<T>>(addr);
            Vector::append(&mut box.thing, thing);
        }else{
            move_to(account, Box<T>{thing})
        }
    }

    spec fun put_all {aborts_if false;}

    // Take last thing from account's box
    public fun take<T>(account: &signer): T acquires Box{
        let addr = Signer::address_of(account);
        assert(exists_at<T>(addr), Errors::invalid_state(EBOX_NOT_EXIST));
        let box = borrow_global_mut<Box<T>>(addr);
        let thing = Vector::pop_back(&mut box.thing);
        if (Vector::is_empty(&box.thing)){
            destroy_empty<T>(addr);
        };
        thing
    }

    spec fun take {
        aborts_if !exists_at<T>(Signer::address_of(account));
        aborts_if !exists<Box<T>>(Signer::address_of(account));
        aborts_if len(global<Box<T>>(Signer::address_of(account)).thing) == 0;
    }

    public fun take_all<T>(account: &signer): vector<T> acquires Box{
        let addr = Signer::address_of(account);
        assert(exists_at<T>(addr), Errors::invalid_state(EBOX_NOT_EXIST));
        let Box{ thing } = move_from<Box<T>>(addr);
        thing
    }

    spec fun take_all {
        aborts_if !exists_at<T>(Signer::address_of(account));
    }

    fun destroy_empty<T>(addr: address) acquires Box{
        let Box{ thing } = move_from<Box<T>>(addr);
        Vector::destroy_empty(thing);
    }

    spec fun destroy_empty {
        aborts_if !exists<Box<T>>(addr);
        aborts_if len(global<Box<T>>(addr).thing) > 0;
    }

}
}