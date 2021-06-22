// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_state_api::StateReaderExt;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::ScriptFunction;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::account_config::{genesis_address, stc_type_tag};
use starcoin_vm_types::sips::{SIP, SIPS};
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use test_helper::dao::dao_vote_test;
use test_helper::executor::*;
use test_helper::Account;

pub const TEST_SIP_10000: &str = r#"
    module {{sender}}::SIP_10000 {
    }
    "#;

#[stest::test]
fn test_sip_flags() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();
    for sip in SIPS.iter() {
        assert!(chain_state.is_activated(*sip)?);
    }

    let sip_10000 = SIP {
        id: 10000,
        module_name: "SIP_10000",
        url: "",
    };

    assert!(!chain_state.is_activated(sip_10000)?);

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModuleV2").unwrap(),
        type_params: vec![],
    });

    let module = compile_modules_with_address(genesis_address(), TEST_SIP_10000)
        .pop()
        .unwrap();
    let package = Package::new_with_module(module)?;
    let package_hash = package.crypto_hash();

    let vote_script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("ModuleUpgradeScripts").unwrap(),
        ),
        Identifier::new("propose_module_upgrade_v2").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(&genesis_address()).unwrap(),
            bcs_ext::to_bytes(&package_hash.to_vec()).unwrap(),
            bcs_ext::to_bytes(&1u64).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
            bcs_ext::to_bytes(&false).unwrap(),
        ],
    );
    let execute_script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("ModuleUpgradeScripts").unwrap(),
        ),
        Identifier::new("submit_module_upgrade_plan").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(alice.address()).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    );
    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag,
        execute_script_function,
        0,
    )?;
    association_execute_should_success(&net, &chain_state, TransactionPayload::Package(package))?;

    assert!(chain_state.is_activated(sip_10000)?);
    Ok(())
}
