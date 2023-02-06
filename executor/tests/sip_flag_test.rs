// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use starcoin_state_api::StateReaderExt;
use starcoin_vm_types::sips::{G_SIPS, SIP};
use test_helper::executor::*;
use test_helper::Account;

pub const TEST_SIP_10000: &str = r#"
    module {{sender}}::SIP_10000 {
    }
    "#;

#[stest::test]
fn test_sip_flags() -> Result<()> {
    let _alice = Account::new();
    let (chain_state, _net) = prepare_genesis();
    for sip in G_SIPS.iter() {
        assert!(chain_state.is_activated(*sip)?);
    }

    let sip_10000 = SIP {
        id: 10000,
        module_name: "SIP_10000",
        url: "",
    };

    assert!(!chain_state.is_activated(sip_10000)?);

    // TODO: test with StarcoinDAO
    // let module = compile_modules_with_address(genesis_address(), TEST_SIP_10000)
    //     .pop()
    //     .unwrap();
    // let package = Package::new_with_module(module)?;
    // let package_hash = package.crypto_hash();
    //
    // let vote_script_function = ScriptFunction::new(
    //     ModuleId::new(
    //         core_code_address(),
    //         Identifier::new("UpgradeModulePlugin").unwrap(),
    //     ),
    //     Identifier::new("create_proposal_entry").unwrap(),
    //     vec![starcoin_dao_type_tag()],
    //     vec![
    //         bcs_ext::to_bytes("upgrade sip").unwrap(),
    //         bcs_ext::to_bytes("upgrade sip").unwrap(),
    //         bcs_ext::to_bytes("upgrade sip").unwrap(),
    //         bcs_ext::to_bytes(&3600000u64).unwrap(),
    //         bcs_ext::to_bytes(&package_hash.to_vec()).unwrap(),
    //         bcs_ext::to_bytes(&1u64).unwrap(),
    //         bcs_ext::to_bytes(&false).unwrap(),
    //     ],
    // );
    // let proposal_id = 1u64;
    // let execute_script_function = ScriptFunction::new(
    //     ModuleId::new(
    //         core_code_address(),
    //         Identifier::new("UpgradeModulePlugin").unwrap(),
    //     ),
    //     Identifier::new("execute_proposal_entry").unwrap(),
    //     vec![starcoin_dao_type_tag()],
    //     vec![bcs_ext::to_bytes(&proposal_id).unwrap()],
    // );
    // starcoin_dao::dao_vote_test(
    //     &alice,
    //     &chain_state,
    //     &net,
    //     vote_script_function,
    //     execute_script_function,
    //     proposal_id,
    // )?;
    //
    // association_execute_should_success(&net, &chain_state, TransactionPayload::Package(package))?;
    //
    // assert!(chain_state.is_activated(sip_10000)?);
    Ok(())
}
