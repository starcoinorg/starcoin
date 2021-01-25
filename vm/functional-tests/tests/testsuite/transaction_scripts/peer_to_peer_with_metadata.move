//! account: alice, 100000 0x1::STC::STC
//! account: bob

//! sender: alice
//! type-args: 0x1::STC::STC
//! args: {{bob}}, x"", 100u128, x""
stdlib_script::peer_to_peer_with_metadata
// check: gas_used
// check: 120571
// check: "Keep(EXECUTED)"

