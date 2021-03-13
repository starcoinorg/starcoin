module MyCounter {
     use 0x1::Signer;

     struct T has key, store {
        value:u64,
     }
     public fun init(account: &signer){
        move_to(account, T{value:0});
     }
     public fun incr(account: &signer) acquires T {
        let counter = borrow_global_mut<T>(Signer::address_of(account));
        counter.value = counter.value + 1;
     }

}