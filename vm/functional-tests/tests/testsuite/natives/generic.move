// Test for Generic Module in Move

module M {
    struct T{
    }
}

// check: EXECUTED


//! new-transaction

script {
use {{default}}::M;
use 0x0::Generic;
use 0x0::Transaction;

fun main() {

    let (address, module_name, struct_name) = Generic::type_of<M::T>();
    Transaction::assert(address == {{default}}, 8001);
    // M
    Transaction::assert(module_name == x"4d", 8002);
    // T
    Transaction::assert(struct_name == x"54", 8003);

}
}

// check: EXECUTED