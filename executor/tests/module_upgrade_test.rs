use anyhow::Result;
use logger::prelude::*;
use starcoin_config::genesis_config::TOTAL_STC_AMOUNT;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_executor::execute_readonly_function;
use starcoin_state_api::{ChainStateReader, StateReaderExt, StateView};
use starcoin_transaction_builder::{build_package_with_stdlib_module, StdLibOptions};
use starcoin_types::access_path::DataPath;
use starcoin_types::account_config::config_change::ConfigChangeEvent;
use starcoin_types::account_config::TwoPhaseUpgradeV2Resource;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::ScriptFunction;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::upgrade::UpgradeEvent;
use starcoin_vm_types::account_config::{association_address, core_code_address, AccountResource};
use starcoin_vm_types::account_config::{genesis_address, stc_type_tag};
use starcoin_vm_types::genesis_config::{ChainId, StdlibVersion};
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::on_chain_config::{MoveLanguageVersion, TransactionPublishOption, Version};
use starcoin_vm_types::on_chain_resource::LinearWithdrawCapability;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE;
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use statedb::ChainStateDB;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use stdlib::{load_upgrade_package, StdlibCompat, STDLIB_VERSIONS};
use test_helper::dao::{
    dao_vote_test, execute_script_on_chain_config, on_chain_config_type_tag, vote_language_version,
};
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
    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag.clone(),
        execute_script_function,
        0,
    )?;
    association_execute_should_success(&net, &chain_state, TransactionPayload::Package(package))?;

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
    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_script_function,
        dao_action_type_tag,
        execute_script_function,
        1,
    )?;
    association_execute_should_success(&net, &chain_state, TransactionPayload::Package(package))?;

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
        StdLibOptions::Compiled(StdlibVersion::Version(3)),
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

    assert!(!read_two_phase_upgrade_v2_resource(&chain_state)?);
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

    assert!(!read_two_phase_upgrade_v2_resource(&chain_state)?);
    Ok(())
}

#[stest::test(timeout = 300)]
fn test_stdlib_upgrade() -> Result<()> {
    let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
    let stdlib_versions = STDLIB_VERSIONS.clone();
    let mut current_version = stdlib_versions[0];
    genesis_config.stdlib_version = current_version;
    let net = ChainNetwork::new_custom(
        "test_stdlib_upgrade".to_string(),
        ChainId::new(100),
        genesis_config,
    )?;
    let chain_state = prepare_customized_genesis(&net);
    let mut proposal_id: u64 = 0;
    let alice = Account::new();

    for new_version in stdlib_versions.into_iter().skip(1) {
        // if upgrade from 7 to later, we need to update language version to 3.
        if let StdlibVersion::Version(7) = current_version {
            dao_vote_test(
                &alice,
                &chain_state,
                &net,
                vote_language_version(&net, 3),
                on_chain_config_type_tag(MoveLanguageVersion::type_tag()),
                execute_script_on_chain_config(&net, MoveLanguageVersion::type_tag(), proposal_id),
                proposal_id,
            )?;
            proposal_id += 1;
        }
        // if upgrade from 10 to later, we need to update language version to 4.
        if let StdlibVersion::Version(10) = current_version {
            dao_vote_test(
                &alice,
                &chain_state,
                &net,
                vote_language_version(&net, 4),
                on_chain_config_type_tag(MoveLanguageVersion::type_tag()),
                execute_script_on_chain_config(&net, MoveLanguageVersion::type_tag(), proposal_id),
                proposal_id,
            )?;
            proposal_id += 1;
        }
        verify_version_state(current_version, &chain_state)?;

        let dao_action_type_tag = new_version.upgrade_module_type_tag();
        let package = match load_upgrade_package(current_version, new_version)? {
            Some(package) => package,
            None => {
                info!(
                    "{:?} is same as {:?}, continue",
                    current_version, new_version
                );
                continue;
            }
        };
        let package_hash = package.crypto_hash();

        let vote_script_function = new_version.propose_module_upgrade_function(
            stc_type_tag(),
            genesis_address(),
            package_hash,
            0,
            !StdlibVersion::compatible_with_previous(&new_version),
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
                bcs_ext::to_bytes(&proposal_id).unwrap(),
            ],
        );
        dao_vote_test(
            &alice,
            &chain_state,
            &net,
            vote_script_function,
            dao_action_type_tag,
            execute_script_function,
            proposal_id,
        )?;

        let output = association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::Package(package),
        )?;
        let contract_event = expect_event::<UpgradeEvent>(&output);
        let _upgrade_event = contract_event.decode_event::<UpgradeEvent>()?;

        let _version_config_event = expect_event::<ConfigChangeEvent<Version>>(&output);

        ext_execute_after_upgrade(new_version, &net, &chain_state)?;
        proposal_id += 1;
        current_version = new_version;
    }

    Ok(())
}

fn ext_execute_after_upgrade(
    version: StdlibVersion,
    net: &ChainNetwork,
    chain_state: &ChainStateDB,
) -> Result<()> {
    match version {
        StdlibVersion::Version(1) => {
            //do nothing
        }
        StdlibVersion::Version(2) => {
            //do nothing
        }
        StdlibVersion::Version(3) => {
            let take_liner_time_capability = ScriptFunction::new(
                ModuleId::new(
                    core_code_address(),
                    Identifier::new("StdlibUpgradeScripts").unwrap(),
                ),
                Identifier::new("take_linear_withdraw_capability").unwrap(),
                vec![],
                vec![],
            );
            association_execute_should_success(
                net,
                chain_state,
                TransactionPayload::ScriptFunction(take_liner_time_capability),
            )?;
        }
        StdlibVersion::Version(6) => {
            let resource = chain_state.get(&AccessPath::new(
                genesis_address(),
                DataPath::Resource(StructTag {
                    address: genesis_address(),
                    module: Identifier::new("Account").unwrap(),
                    name: Identifier::new("SignerDelegated").unwrap(),
                    type_params: vec![],
                }),
            ))?;
            assert!(resource.is_some());
            let genesis_account = chain_state
                .get_account_resource(genesis_address())?
                .unwrap();
            assert!(
                genesis_account.has_delegated_key_rotation_capability(),
                "expect 0x1 has no key rotation capability"
            );
            println!("genesis: {:?}", &genesis_account);
            assert_eq!(
                genesis_account.authentication_key(),
                &AccountResource::CONTRACT_AUTH_KEY
            );
        }
        StdlibVersion::Version(7) => {
            let version_resource = chain_state.get_on_chain_config::<MoveLanguageVersion>()?;
            assert!(version_resource.is_some());
            let version = version_resource.unwrap();
            assert_eq!(version.major, 2, "expect language version is 2");
            let genesis_nft_info = chain_state.get(&AccessPath::new(
                genesis_address(),
                DataPath::Resource(StructTag {
                    address: genesis_address(),
                    module: Identifier::new("GenesisNFT").unwrap(),
                    name: Identifier::new("GenesisNFTInfo").unwrap(),
                    type_params: vec![],
                }),
            ))?;
            assert!(
                genesis_nft_info.is_some(),
                "expect 0x1::GenesisNFT::GenesisNFTInfo in global storage, but go none."
            );
        }
        _ => {
            //do nothing.
        }
    }
    Ok(())
}

fn verify_version_state<R>(version: StdlibVersion, chain_state: &R) -> Result<()>
where
    R: ChainStateReader,
{
    match version {
        StdlibVersion::Version(1) => {
            //TODO
        }
        StdlibVersion::Version(2) => {
            assert!(
                chain_state.get_stc_treasury()?.is_none(),
                "expect treasury is none."
            );
            assert!(!read_two_phase_upgrade_v2_resource(chain_state)?);
        }
        StdlibVersion::Version(3) => {
            assert!(
                chain_state.get_stc_treasury()?.is_some(),
                "expect treasury is some."
            );
            assert_eq!(
                chain_state.get_stc_info().unwrap().unwrap().total_value,
                TOTAL_STC_AMOUNT.scaling()
            );
            let withdraw_cap = chain_state
                .get_resource_by_access_path::<LinearWithdrawCapability>(
                    LinearWithdrawCapability::resource_path_for(
                        association_address(),
                        STC_TOKEN_CODE.clone().try_into()?,
                    ),
                )?;
            assert!(
                withdraw_cap.is_some(),
                "expect LinearWithdrawCapability exist at association_address"
            );
        }
        _ => {
            //do nothing.
        }
    }
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

    assert_eq!(read_foo(&chain_state), 1);
    Ok(())
}

fn read_two_phase_upgrade_v2_resource<R>(state_reader: &R) -> Result<bool>
where
    R: ChainStateReader,
{
    Ok(state_reader
        .get_resource::<TwoPhaseUpgradeV2Resource>(genesis_address())?
        .map(|tpu| tpu.enforced())
        .unwrap_or(false))
}

fn read_foo(state_view: &dyn StateView) -> u8 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("M").unwrap()),
        &Identifier::new("foo").unwrap(),
        vec![],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}
