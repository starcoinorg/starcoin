// Test for Generic Module in Move
//! account: alice
//! account: bob

//! sender: alice
module M {
    struct T{
    }
}

// check: EXECUTED


//! new-transaction
//! sender: bob
script {
use {{alice}}::M;
use 0x0::Generic;
use 0x0::Transaction;

fun main() {

    let (address, module_name, struct_name) = Generic::type_of<M::T>();
    Transaction::assert(address == {{alice}}, 8001);
    // M
    Transaction::assert(module_name == x"4d", 8002);
    // T
    Transaction::assert(struct_name == x"54", 8003);

}
}

// check: EXECUTED