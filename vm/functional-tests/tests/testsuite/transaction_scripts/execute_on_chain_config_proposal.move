//! account: alice, 15925680000000000 0x1::STC::STC
//! account: bob, 15925680000000000 0x1::STC::STC

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 86400000

//! new-transaction
//! sender: alice
//! args: x"", false, 0
stdlib_script::propose_update_txn_publish_option
// check: gas_used
// check: 181217
// check: "Keep(EXECUTED)"

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 87000000

//! new-transaction
//! sender: bob
//! type-args: 0x1::STC::STC, 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::TransactionPublishOption::TransactionPublishOption>
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
//! type-args: 0x1::STC::STC, 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::TransactionPublishOption::TransactionPublishOption>
//! args: {{alice}}, 0
stdlib_script::queue_proposal_action
// check: gas_used
// check: 54457
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//! type-args: 0x1::STC::STC, 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::TransactionPublishOption::TransactionPublishOption>
//! args: {{alice}}, 0
stdlib_script::unstake_vote
// check: gas_used
// check: 114622
// check: "Keep(EXECUTED)"

//! block-prologue
//! author: genesis
//! block-number: 4
//! block-time: 130000000

//! new-transaction
//! sender: alice
//! type-args: 0x1::TransactionPublishOption::TransactionPublishOption
//! args: 0
stdlib_script::execute_on_chain_config_proposal
// check: gas_used
// check: 97770
// check: "Keep(EXECUTED)"