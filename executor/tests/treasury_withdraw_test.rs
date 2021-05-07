// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_state_api::StateReaderExt;
use starcoin_types::account_config::stc_type_tag;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::on_chain_resource::dao::WithdrawToken;
use starcoin_vm_types::on_chain_resource::LinearWithdrawCapability;
use starcoin_vm_types::token::stc::{STCUnit, STC_TOKEN_CODE};
use starcoin_vm_types::transaction::ScriptFunction;
use test_helper::dao::dao_vote_test;
use test_helper::executor::prepare_genesis;
use test_helper::Account;

#[stest::test]
fn test_treasury_withdraw() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();
    let action_type_tag = WithdrawToken::type_tag();

    let withdraw_amount = STCUnit::STC.value_of(1000);
    let period = 1000u64;
    let proposal_id = 0u64;

    let vote_script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TreasuryScripts").unwrap(),
        ),
        Identifier::new("propose_withdraw").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(alice.address()).unwrap(),
            bcs_ext::to_bytes(&withdraw_amount.scaling()).unwrap(),
            bcs_ext::to_bytes(&period).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    );

    let execute_script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TreasuryScripts").unwrap(),
        ),
        Identifier::new("execute_withdraw_proposal").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(alice.address()).unwrap(),
            bcs_ext::to_bytes(&proposal_id).unwrap(),
        ],
    );

    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_script_function,
        action_type_tag,
        execute_script_function,
        proposal_id,
    )?;

    let cap = chain_state.get_resource_by_access_path::<LinearWithdrawCapability>(
        LinearWithdrawCapability::resource_path_for(*alice.address(), STC_TOKEN_CODE.clone()),
    )?;
    assert!(cap.is_some(), "expect LinearWithdrawCapability exist.");
    let cap = cap.unwrap();
    assert_eq!(cap.total, withdraw_amount.scaling());
    assert_eq!(cap.period, period);
    Ok(())
}
