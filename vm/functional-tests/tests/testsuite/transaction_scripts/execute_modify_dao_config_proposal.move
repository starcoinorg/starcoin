//! account: alice, 15925680000000000 0x1::STC::STC
//! account: bob, 15925680000000000 0x1::STC::STC

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 86400000

//! new-transaction
//! sender: alice
//! type-args: 0x1::STC::STC
//! args: 86400000, 0, 50u8, 0, 0
stdlib_script::propose_modify_dao_config
// check: gas_used
// check: 186513
// check: "Keep(EXECUTED)"

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 87000000

//! new-transaction
//! sender: bob
//! type-args: 0x1::STC::STC, 0x1::ModifyDaoConfigProposal::DaoConfigUpdate
//! args: {{alice}}, 0, true, 3981420001000000u128
stdlib_script::cast_vote
// check: gas_used
// check: 170700
// check: "Keep(EXECUTED)"


//! block-prologue
//! author: genesis
//! block-number: 3
//! block-time: 110000000

//! new-transaction
//! sender: bob
//! type-args: 0x1::STC::STC, 0x1::ModifyDaoConfigProposal::DaoConfigUpdate
//! args: {{alice}}, 0
stdlib_script::queue_proposal_action
// check: gas_used
// check: 54457
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//! type-args: 0x1::STC::STC, 0x1::ModifyDaoConfigProposal::DaoConfigUpdate
//! args: {{alice}}, 0
stdlib_script::unstake_vote
// check: gas_used
// check: 114622
// check: "Keep(EXECUTED)"

//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 250000000

//! new-transaction
//! sender: bob
//! type-args: 0x1::STC::STC
//! args: {{alice}}, 0
stdlib_script::execute_modify_dao_config_proposal
// check: gas_used
// check: 149562
// check: "Keep(EXECUTED)"