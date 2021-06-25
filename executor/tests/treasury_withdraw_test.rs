// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::genesis_config::TOTAL_STC_AMOUNT;
use starcoin_executor::account::create_account_txn_sent_as_association;
use starcoin_state_api::StateReaderExt;
use starcoin_transaction_builder::DEFAULT_MAX_GAS_AMOUNT;
use starcoin_types::account_config::stc_type_tag;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::on_chain_resource::dao::WithdrawToken;
use starcoin_vm_types::on_chain_resource::LinearWithdrawCapability;
use starcoin_vm_types::token::stc::{STCUnit, STC_TOKEN_CODE, STC_TOKEN_CODE_STR};
use starcoin_vm_types::transaction::{
    RawUserTransaction, ScriptFunction, Transaction, TransactionPayload,
};
use starcoin_vm_types::vm_status::KeptVMStatus;
use std::convert::TryInto;
use test_helper::dao::dao_vote_test;
use test_helper::executor::{execute_and_apply, prepare_genesis};
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
        LinearWithdrawCapability::resource_path_for(
            *alice.address(),
            STC_TOKEN_CODE.clone().try_into()?,
        ),
    )?;
    assert!(cap.is_some(), "expect LinearWithdrawCapability exist.");
    let cap = cap.unwrap();
    assert_eq!(cap.total, withdraw_amount.scaling());
    assert_eq!(cap.period, period);
    Ok(())
}

#[stest::test]
fn test_treasury_withdraw_too_many() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &alice,
        0,
        STCUnit::STC.value_of(100).scaling(),
        1,
        &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let withdraw_amount = TOTAL_STC_AMOUNT.scaling() / 2;
    let period = 1000u64;

    let vote_script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TreasuryScripts").unwrap(),
        ),
        Identifier::new("propose_withdraw").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(alice.address()).unwrap(),
            bcs_ext::to_bytes(&withdraw_amount).unwrap(),
            bcs_ext::to_bytes(&period).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    );

    let raw_txn = RawUserTransaction::new(
        *alice.address(),
        0,
        TransactionPayload::ScriptFunction(vote_script_function),
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        3600,
        net.chain_id(),
        STC_TOKEN_CODE_STR.to_string(),
    );
    let txn2 = alice.sign_txn(raw_txn);
    let output1 = execute_and_apply(&chain_state, Transaction::UserTransaction(txn2));
    //println!("{:?}", output1.status().status().unwrap());
    assert!(
        matches!(
            output1.status().status().unwrap(),
            KeptVMStatus::MoveAbort(_, 26375)
        ),
        "expect move abort"
    );
    Ok(())
}
