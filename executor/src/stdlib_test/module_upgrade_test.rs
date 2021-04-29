use crate::execute_readonly_function;
use anyhow::{format_err, Result};
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_state_api::StateView;
use starcoin_transaction_builder::{
    build_package_with_stdlib_module, build_stdlib_package_for_test, StdLibOptions,
};
use starcoin_types::account_config::{
    access_path_for_two_phase_upgrade_v2, TwoPhaseUpgradeV2Resource,
};
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::ScriptFunction;
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::account_config::{genesis_address, stc_type_tag};
use starcoin_vm_types::genesis_config::{ChainId, StdlibVersion};
use starcoin_vm_types::on_chain_config::TransactionPublishOption;
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use starcoin_vm_types::values::VMValueCast;
use std::fs::File;
use std::io::Read;
use test_helper::dao::dao_vote_test;
use test_helper::executor::*;
use test_helper::Account;

#[stest::test]
fn test_dao_upgrade_module() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModuleV2").unwrap(),
        type_params: vec![],
    });
    let module = compile_modules_with_address(genesis_address(), TEST_MODULE)
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
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag,
        execute_script_function,
        0,
    )?;
    association_execute(&net, &chain_state, TransactionPayload::Package(package))?;

    assert_eq!(read_foo(&chain_state), 1);
    Ok(())
}

#[stest::test]
fn test_dao_upgrade_module_enforced() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModuleV2").unwrap(),
        type_params: vec![],
    });
    let module = compile_modules_with_address(genesis_address(), TEST_MODULE)
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
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag.clone(),
        execute_script_function,
        0,
    )?;
    association_execute(&net, &chain_state, TransactionPayload::Package(package))?;

    assert_eq!(read_foo(&chain_state), 1);

    // test upgrade module enforced
    let alice = Account::new();
    let module = compile_modules_with_address(genesis_address(), TEST_MODULE_1)
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
            bcs_ext::to_bytes(&2u64).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
            bcs_ext::to_bytes(&true).unwrap(),
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
            bcs_ext::to_bytes(&1u64).unwrap(),
        ],
    );
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag,
        execute_script_function,
        1,
    )?;
    association_execute(&net, &chain_state, TransactionPayload::Package(package))?;

    assert_eq!(read_foo(&chain_state), 2);
    Ok(())
}

#[stest::test]
fn test_init_script() -> Result<()> {
    let alice = Account::new();
    let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
    genesis_config.stdlib_version = StdlibVersion::Version(1);
    let net = ChainNetwork::new_custom(
        "init_script_test".to_string(),
        ChainId::new(100),
        genesis_config,
    )?;
    let chain_state = prepare_customized_genesis(&net);

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModule").unwrap(),
        type_params: vec![],
    });

    let init_script = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("PackageTxnManager").unwrap(),
        ),
        Identifier::new("convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2").unwrap(),
        vec![],
        vec![bcs_ext::to_bytes(&genesis_address()).unwrap()],
    );

    let module_names = vec!["Errors", "PackageTxnManager"];
    let package = build_package_with_stdlib_module(
        StdLibOptions::Compiled(StdlibVersion::Latest),
        module_names,
        Some(init_script),
    )?;
    let package_hash = package.crypto_hash();

    let vote_script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("ModuleUpgradeScripts").unwrap(),
        ),
        Identifier::new("propose_module_upgrade").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(&genesis_address()).unwrap(),
            bcs_ext::to_bytes(&package_hash.to_vec()).unwrap(),
            bcs_ext::to_bytes(&1u64).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
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
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag,
        execute_script_function,
        0,
    )?;
    association_execute(&net, &chain_state, TransactionPayload::Package(package))?;

    assert_eq!(read_two_phase_upgrade_v2_resource(&chain_state)?, false);
    Ok(())
}

#[stest::test]
fn test_upgrade_stdlib_with_incremental_package() -> Result<()> {
    let alice = Account::new();
    let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
    genesis_config.stdlib_version = StdlibVersion::Version(1);
    let net = ChainNetwork::new_custom(
        "test_stdlib_upgrade".to_string(),
        ChainId::new(100),
        genesis_config,
    )?;
    let chain_state = prepare_customized_genesis(&net);

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModule").unwrap(),
        type_params: vec![],
    });
    let path = std::path::PathBuf::from("../vm/stdlib/compiled/2/1-2/stdlib.blob")
        .canonicalize()
        .unwrap();
    let mut bytes = vec![];
    File::open(path)?.read_to_end(&mut bytes)?;
    let package: Package = bcs_ext::from_bytes(&bytes)?;

    let package_hash = package.crypto_hash();

    let vote_script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("ModuleUpgradeScripts").unwrap(),
        ),
        Identifier::new("propose_module_upgrade").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(&genesis_address()).unwrap(),
            bcs_ext::to_bytes(&package_hash.to_vec()).unwrap(),
            bcs_ext::to_bytes(&1u64).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
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
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag,
        execute_script_function,
        0,
    )?;
    association_execute(&net, &chain_state, TransactionPayload::Package(package))?;

    assert_eq!(read_two_phase_upgrade_v2_resource(&chain_state)?, false);
    Ok(())
}

#[stest::test]
fn test_stdlib_upgrade() -> Result<()> {
    let alice = Account::new();
    let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
    genesis_config.stdlib_version = StdlibVersion::Version(1);
    let net = ChainNetwork::new_custom(
        "test_stdlib_upgrade".to_string(),
        ChainId::new(100),
        genesis_config,
    )?;
    let chain_state = prepare_customized_genesis(&net);

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModule").unwrap(),
        type_params: vec![],
    });

    let init_script = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("PackageTxnManager").unwrap(),
        ),
        Identifier::new("convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2").unwrap(),
        vec![],
        vec![bcs_ext::to_bytes(&genesis_address()).unwrap()],
    );

    let package = build_stdlib_package_for_test(
        StdLibOptions::Compiled(StdlibVersion::Latest),
        Some(init_script),
    )?;
    let package_hash = package.crypto_hash();

    let vote_script_function = ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("ModuleUpgradeScripts").unwrap(),
        ),
        Identifier::new("propose_module_upgrade").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(&genesis_address()).unwrap(),
            bcs_ext::to_bytes(&package_hash.to_vec()).unwrap(),
            bcs_ext::to_bytes(&1u64).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
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
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag,
        execute_script_function,
        0,
    )?;
    association_execute(&net, &chain_state, TransactionPayload::Package(package))?;

    assert_eq!(read_two_phase_upgrade_v2_resource(&chain_state)?, false);
    Ok(())
}

#[stest::test]
fn test_upgrade_stdlib_with_disallowed_publish_option() -> Result<()> {
    let alice = Account::new();
    let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
    genesis_config.publishing_option = TransactionPublishOption::locked();
    let net = ChainNetwork::new_custom(
        "test_stdlib_upgrade".to_string(),
        ChainId::new(100),
        genesis_config,
    )?;
    let chain_state = prepare_customized_genesis(&net);

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModuleV2").unwrap(),
        type_params: vec![],
    });
    let module = compile_modules_with_address(genesis_address(), TEST_MODULE)
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
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag,
        execute_script_function,
        0,
    )?;
    association_execute(&net, &chain_state, TransactionPayload::Package(package))?;

    assert_eq!(read_foo(&chain_state), 1);
    Ok(())
}

fn read_two_phase_upgrade_v2_resource(state_view: &dyn StateView) -> Result<bool> {
    let two_phase_upgrade_v2_path = access_path_for_two_phase_upgrade_v2(genesis_address());
    match state_view.get(&two_phase_upgrade_v2_path)? {
        Some(data) => Ok(bcs_ext::from_bytes::<TwoPhaseUpgradeV2Resource>(&data)?.enforced()),
        _ => Err(format_err!("read two phase upgrade resource fail.")),
    }
}

fn read_foo(state_view: &dyn StateView) -> u8 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("M").unwrap()),
        &Identifier::new("foo").unwrap(),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}
