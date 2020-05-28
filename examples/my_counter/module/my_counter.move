module MyCounter {
     use 0x0::Transaction;
     resource struct T {
        value:u64,
     }
     public fun init(){
        move_to_sender(T{value:0});
     }
     public fun incr() acquires T {
        let counter = borrow_global_mut<T>(Transaction::sender());
        counter.value = counter.value + 1;
     }

}