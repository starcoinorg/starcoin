//! account: alice, 100000 0x1::STC::STC
//! account: bob

//! sender: alice
//! type-args: 0x1::STC::STC
//! args: {{bob}}, x"", 100u128
stdlib_script::peer_to_peer
// check: gas_used
// check: 124205
// check: "Keep(EXECUTED)"

