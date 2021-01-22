//! account: alice, 100000 0x1::STC::STC
//! account: bob

//! sender: alice
//! type-args: 0x1::STC::STC
//! args: 0x75995fa86f8ebc0b0819ebf80abc0ee6, x"fb51f08c8e63ed9f4eac340b25d1b01d75995fa86f8ebc0b0819ebf80abc0ee6", 100u128
stdlib_script::create_account
// check: gas_used
// check: 1030404
// check: "Keep(EXECUTED)"