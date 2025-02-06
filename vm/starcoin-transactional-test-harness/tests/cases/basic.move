//# init -n test

//# faucet --addr alice --amount 10000000000000000

//
// //# publish
// module alice::basic_module {
//     use std::option;
//     use std::signer;
//     use std::string;
//
//     use starcoin_framework::coin;
//     use starcoin_framework::fungible_asset::{create_store, FungibleStore};
//     use starcoin_framework::object;
//     use starcoin_framework::starcoin_coin::STC;
//     use starcoin_std::debug;
//
//     #[resource_group_member(group = starcoin_framework::object::ObjectGroup)]
//     struct Sample has key {
//         value: u64
//     }
//
//     struct ObjectWrap has key {
//         obj_addr: address,
//         store: object::Object<FungibleStore>
//     }
//
//     public fun create_sample(account: &signer, value: u64) {
//         debug::print(&string::utf8(b"alice::basic_module::create_sample | 1"));
//
//         let ref = object::create_object_from_account(account);
//         move_to(&object::generate_signer(&ref), Sample {
//             value
//         });
//
//         debug::print(&string::utf8(b"alice::basic_module::create_sample | 2"));
//
//         let metadata = coin::paired_metadata<STC>();
//
//         debug::print(&string::utf8(b"alice::basic_module::create_sample | 3"));
//
//         let store = create_store(&ref, option::destroy_some(metadata));
//
//         debug::print(&string::utf8(b"alice::basic_module::create_sample | 4"));
//
//         move_to(account, ObjectWrap {
//             obj_addr: object::address_from_constructor_ref(&ref),
//             store,
//         });
//
//         debug::print(&string::utf8(b"alice::basic_module::create_sample | 5"));
//     }
//
//     public fun check_value(account: &signer): u64 acquires ObjectWrap, Sample {
//         let obj_wrap = borrow_global<ObjectWrap>(signer::address_of(account));
//         borrow_global<Sample>(obj_wrap.obj_addr).value
//     }
// }

//# run --signers alice
script {
    use std::option;
    use starcoin_framework::object;
    use starcoin_framework::coin;
    use starcoin_framework::starcoin_coin::STC;

    fun test_metadata(_account: &signer) {
        let metadata = coin::paired_metadata<STC>();
        assert!(option::is_some(&metadata), 10000);
        let metdata_obj = option::destroy_some(metadata);
        assert!(object::is_object(object::object_address(&metdata_obj)), 10001);
    }
}


// //# run --signers alice
// script {
//     use alice::basic_module;
//
//     fun test_create_object(account: &signer) {
//         basic_module::create_sample(account, 10);
//     }
// }
//
//
// //# run --signers alice
// script {
//     use alice::basic_module;
//
//     fun test_create_object(account: &signer) {
//         assert!(basic_module::check_value(account) == 10, 10010);
//     }
// }
