address 0x1 {
// Provider a account based vector for save resource.
module Box {
    use 0x1::Signer;
    use 0x1::ErrorCode;
    use 0x1::Vector;

    resource struct Box<T>{
        thing:vector<T>,
    }

    fun EBOX_NOT_EXIST(): u64{
        ErrorCode::ECODE_BASE() + 1
    }

    public fun exists_at<T>(addr: address): bool{
        exists<Box<T>>(addr)
    }

    public fun length<T>(addr: address): u64 acquires Box{
        if (exists_at<T>(addr)) {
            let box = borrow_global<Box<T>>(addr);
            Vector::length(&box.thing)
        }else{
           0
        }
    }

    // Put thing to account's box last postion.
    public fun put<T>(account: &signer, thing: T) acquires Box{
        let addr = Signer::address_of(account);
        if (exists_at<T>(addr)) {
            let box = borrow_global_mut<Box<T>>(addr);
            Vector::push_back(&mut box.thing, thing);
        }else{
            move_to(account, Box<T>{thing: Vector::singleton(thing)})
        }
    }

    public fun put_all<T>(account: &signer, thing: vector<T>) acquires Box{
        let addr = Signer::address_of(account);
        if (exists_at<T>(addr)) {
            let box = borrow_global_mut<Box<T>>(addr);
            Vector::append(&mut box.thing, thing);
        }else{
            move_to(account, Box<T>{thing})
        }
    }

    // Take last thing from account's box
    public fun take<T>(account: &signer): T acquires Box{
        let addr = Signer::address_of(account);
        assert(exists_at<T>(addr), EBOX_NOT_EXIST());
        let box = borrow_global_mut<Box<T>>(addr);
        let thing = Vector::pop_back(&mut box.thing);
        if (Vector::is_empty(&box.thing)){
            destory_empty<T>(addr);
        };
        thing
    }

    public fun take_all<T>(account: &signer): vector<T> acquires Box{
        let addr = Signer::address_of(account);
        assert(exists_at<T>(addr), EBOX_NOT_EXIST());
        let Box{ thing } = move_from<Box<T>>(addr);
        thing
    }

    fun destory_empty<T>(addr: address) acquires Box{
        let Box{ thing } = move_from<Box<T>>(addr);
        Vector::destroy_empty(thing);
    }

}
}