use anyhow::Result;
use starcoin_config::genesis_config::G_TOTAL_STC_AMOUNT;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_logger::prelude::*;
use starcoin_state_api::{ChainStateReader, StateReaderExt, StateView};
use starcoin_statedb::ChainStateDB;
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
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::token::stc::G_STC_TOKEN_CODE;
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use stdlib::{load_upgrade_package, StdlibCompat, G_STDLIB_VERSIONS};
use test_helper::dao::{
    dao_vote_test, execute_script_on_chain_config, on_chain_config_type_tag, vote_language_version,
};
use test_helper::executor::*;
use test_helper::starcoin_dao;
use test_helper::Account;

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

    let dao_action_type_tag = TypeTag::Struct(Box::new(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModule").unwrap(),
        type_params: vec![],
    }));

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

    let dao_action_type_tag = TypeTag::Struct(Box::new(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModule").unwrap(),
        type_params: vec![],
    }));
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
    let stdlib_versions = G_STDLIB_VERSIONS.clone();
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
                vote_language_version(&net, 6),
                on_chain_config_type_tag(MoveLanguageVersion::type_tag()),
                execute_script_on_chain_config(&net, MoveLanguageVersion::type_tag(), proposal_id),
                proposal_id,
            )?;
            proposal_id += 1;
        }
        // if upgrade from 11 to later, we need to update language version to 6.
        if let StdlibVersion::Version(11) = current_version {
            dao_vote_test(
                &alice,
                &chain_state,
                &net,
                vote_language_version(&net, 6),
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

// this is daospace-v12 starcoin-framework
// https://github.com/starcoinorg/starcoin-framework/releases/tag/daospace-v12
// in starcoin master we don't use it
#[ignore]
#[stest::test(timeout = 3000)]
fn test_stdlib_upgrade_since_v12() -> Result<()> {
    let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
    let stdlib_versions = G_STDLIB_VERSIONS.clone();
    let mut current_version = stdlib_versions[0];
    genesis_config.stdlib_version = StdlibVersion::Version(12);
    let net = ChainNetwork::new_custom(
        "test_stdlib_upgrade".to_string(),
        ChainId::new(100),
        genesis_config,
    )?;
    let chain_state = prepare_customized_genesis(&net);
    let mut proposal_id: u64 = 1; // 1-based
    let alice = Account::new();

    for new_version in stdlib_versions.into_iter().skip(1) {
        if current_version < StdlibVersion::Version(12) {
            current_version = new_version;
            continue;
        }

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

        let starcoin_dao_type = TypeTag::Struct(Box::new(StructTag {
            address: genesis_address(),
            module: Identifier::new("StarcoinDAO").unwrap(),
            name: Identifier::new("StarcoinDAO").unwrap(),
            type_params: vec![],
        }));
        let vote_script_function = new_version.propose_module_upgrade_function_since_v12(
            starcoin_dao_type.clone(),
            "upgrade stdlib",
            "upgrade stdlib",
            "upgrade stdlib",
            3600000,
            package_hash,
            !StdlibVersion::compatible_with_previous(&new_version),
        );

        let execute_script_function = ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("UpgradeModulePlugin").unwrap(),
            ),
            Identifier::new("execute_proposal_entry").unwrap(),
            vec![starcoin_dao_type],
            vec![bcs_ext::to_bytes(&proposal_id).unwrap()],
        );
        starcoin_dao::dao_vote_test(
            &alice,
            &chain_state,
            &net,
            vote_script_function,
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
            let resource = chain_state.get_state_value(&StateKey::AccessPath(AccessPath::new(
                genesis_address(),
                DataPath::Resource(StructTag {
                    address: genesis_address(),
                    module: Identifier::new("Account").unwrap(),
                    name: Identifier::new("SignerDelegated").unwrap(),
                    type_params: vec![],
                }),
            )))?;
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
            let genesis_nft_info =
                chain_state.get_state_value(&StateKey::AccessPath(AccessPath::new(
                    genesis_address(),
                    DataPath::Resource(StructTag {
                        address: genesis_address(),
                        module: Identifier::new("GenesisNFT").unwrap(),
                        name: Identifier::new("GenesisNFTInfo").unwrap(),
                        type_params: vec![],
                    }),
                )))?;
            assert!(
                genesis_nft_info.is_some(),
                "expect 0x1::GenesisNFT::GenesisNFTInfo in global storage, but go none."
            );
        }

        // this is old daospace-v12 starcoin-framework,
        // https://github.com/starcoinorg/starcoin-framework/releases/tag/daospace-v12
        // master don't use it
        // StdlibVersion::Version(12) => {
        //     // New resources at genesis_account.
        //     assert_genesis_resouce_exist(chain_state, "Block", "Checkpoints", vec![]);
        //     assert_genesis_resouce_exist(chain_state, "DAORegistry", "DAORegistry", vec![]);
        //     assert_genesis_resouce_exist(
        //         chain_state,
        //         "DAOExtensionPoint",
        //         "NFTMintCapHolder",
        //         vec![],
        //     );
        //     assert_genesis_resouce_exist(chain_state, "DAOExtensionPoint", "Registry", vec![]);
        //     assert_genesis_resouce_exist(
        //         chain_state,
        //         "DAOExtensionPoint",
        //         "RegistryEventHandlers",
        //         vec![],
        //     );
        //     assert_genesis_resouce_exist(
        //         chain_state,
        //         "DAOPluginMarketplace",
        //         "PluginRegistry",
        //         vec![],
        //     );
        //     assert_genesis_resouce_exist(
        //         chain_state,
        //         "DAOPluginMarketplace",
        //         "RegistryEventHandlers",
        //         vec![],
        //     );
        //     assert_genesis_resouce_exist(
        //         chain_state,
        //         "DAOPluginMarketplace",
        //         "PluginRegistry",
        //         vec![],
        //     );
        //
        //     // DAOSpace plugins
        //     let plugin_names = vec![
        //         "AnyMemberPlugin",
        //         "ConfigProposalPlugin",
        //         "GrantProposalPlugin",
        //         "InstallPluginProposalPlugin",
        //         "MemberProposalPlugin",
        //         "MintProposalPlugin",
        //         "StakeToSBTPlugin",
        //         "UpgradeModulePlugin",
        //         "GasOracleProposalPlugin",
        //         "TreasuryPlugin",
        //     ];
        //     plugin_names.into_iter().for_each(|name| {
        //         let any_member_tag = TypeTag::Struct(StructTag {
        //             address: genesis_address(),
        //             module: Identifier::new(name).unwrap(),
        //             name: Identifier::new(name).unwrap(),
        //             type_params: vec![],
        //         });
        //         assert_genesis_resouce_exist(
        //             chain_state,
        //             "DAOPluginMarketplace",
        //             "PluginEntry",
        //             vec![any_member_tag.clone()],
        //         );
        //         assert_genesis_resouce_exist(
        //             chain_state,
        //             "DAOPluginMarketplace",
        //             "PluginEventHandlers",
        //             vec![any_member_tag],
        //         );
        //     });
        //
        //     // New resources of StarcoinDAO
        //     assert_genesis_resouce_exist(chain_state, "DAOAccount", "DAOAccount", vec![]);
        //     vec![
        //         "InstallPluginProposalPlugin",
        //         "UpgradeModulePlugin",
        //         "ConfigProposalPlugin",
        //         "StakeToSBTPlugin",
        //         "GasOracleProposalPlugin",
        //         "TreasuryPlugin",
        //     ]
        //     .into_iter()
        //     .for_each(|name| {
        //         assert_genesis_resouce_exist(
        //             chain_state,
        //             "DAOSpace",
        //             "InstalledPluginInfo",
        //             vec![TypeTag::Struct(StructTag {
        //                 address: genesis_address(),
        //                 module: Identifier::new(name).unwrap(),
        //                 name: Identifier::new(name).unwrap(),
        //                 type_params: vec![],
        //             })],
        //         )
        //     });
        //     assert_genesis_resouce_exist(
        //         chain_state,
        //         "TreasuryPlugin",
        //         "WithdrawCapabilityHolder",
        //         vec![TypeTag::Struct(StructTag {
        //             address: genesis_address(),
        //             module: Identifier::new("STC").unwrap(),
        //             name: Identifier::new("STC").unwrap(),
        //             type_params: vec![],
        //         })],
        //     );
        //
        //     // DAOCustomConfigModifyCapHolder of StarcoinDAO
        //     vec![
        //         "TransactionPublishOption",
        //         "VMConfig",
        //         "ConsensusConfig",
        //         "RewardConfig",
        //         "TransactionTimeoutConfig",
        //         "LanguageVersion",
        //     ]
        //     .into_iter()
        //     .for_each(|name| {
        //         assert_genesis_resouce_exist(
        //             chain_state,
        //             "DAOSpace",
        //             "DAOCustomConfigModifyCapHolder",
        //             vec![
        //                 TypeTag::Struct(StructTag {
        //                     address: genesis_address(),
        //                     module: Identifier::new("StarcoinDAO").unwrap(),
        //                     name: Identifier::new("StarcoinDAO").unwrap(),
        //                     type_params: vec![],
        //                 }),
        //                 TypeTag::Struct(StructTag {
        //                     address: genesis_address(),
        //                     module: Identifier::new(name).unwrap(),
        //                     name: Identifier::new(name).unwrap(),
        //                     type_params: vec![],
        //                 }),
        //             ],
        //         );
        //     });
        //
        //     // Removed old DAO resources.
        //     vec![
        //         ("ModifyDaoConfigProposal", "DaoConfigModifyCapability"),
        //         ("UpgradeModuleDaoProposal", "UpgradeModuleDaoProposal"),
        //         ("TreasuryWithdrawDaoProposal", "WrappedWithdrawCapability"),
        //     ]
        //     .into_iter()
        //     .for_each(|(module, name)| {
        //         assert_genesis_resouce_not_exist(
        //             chain_state,
        //             module,
        //             name,
        //             vec![TypeTag::Struct(StructTag {
        //                 address: genesis_address(),
        //                 module: Identifier::new("STC").unwrap(),
        //                 name: Identifier::new("STC").unwrap(),
        //                 type_params: vec![],
        //             })],
        //         )
        //     });
        //
        //     vec![
        //         "TransactionPublishOption",
        //         "VMConfig",
        //         "ConsensusConfig",
        //         "RewardConfig",
        //         "TransactionTimeoutConfig",
        //         "LanguageVersion",
        //     ]
        //     .into_iter()
        //     .for_each(|name| {
        //         assert_genesis_resouce_not_exist(
        //             chain_state,
        //             "OnChainConfigDao",
        //             "WrappedConfigModifyCapability",
        //             vec![
        //                 TypeTag::Struct(StructTag {
        //                     address: genesis_address(),
        //                     module: Identifier::new("STC").unwrap(),
        //                     name: Identifier::new("STC").unwrap(),
        //                     type_params: vec![],
        //                 }),
        //                 TypeTag::Struct(StructTag {
        //                     address: genesis_address(),
        //                     module: Identifier::new(name).unwrap(),
        //                     name: Identifier::new(name).unwrap(),
        //                     type_params: vec![],
        //                 }),
        //             ],
        //         );
        //     });
        // }
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
                G_TOTAL_STC_AMOUNT.scaling()
            );
            let withdraw_cap = chain_state
                .get_resource_by_access_path::<LinearWithdrawCapability>(
                    LinearWithdrawCapability::resource_path_for(
                        association_address(),
                        G_STC_TOKEN_CODE.clone().try_into()?,
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
    let _alice = Account::new();
    let mut genesis_config = BuiltinNetworkID::Test.genesis_config().clone();
    genesis_config.publishing_option = TransactionPublishOption::locked();
    let net = ChainNetwork::new_custom(
        "test_stdlib_upgrade".to_string(),
        ChainId::new(100),
        genesis_config,
    )?;
    let _chain_state = prepare_customized_genesis(&net);

    // TODO: test with StarcoinDAO with stdlib v12
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

#[allow(dead_code)]
fn assert_genesis_resouce_exist(
    chain_state: &ChainStateDB,
    module: &str,
    name: &str,
    type_params: Vec<TypeTag>,
) {
    let checkpoint = chain_state
        .get_state_value(&StateKey::AccessPath(AccessPath::new(
            genesis_address(),
            DataPath::Resource(StructTag {
                address: genesis_address(),
                module: Identifier::new(module).unwrap(),
                name: Identifier::new(name).unwrap(),
                type_params,
            }),
        )))
        .unwrap();
    assert!(
        checkpoint.is_some(),
        "expect genesis_account has resource 0x1::{:?}::{:?}, but got none.",
        module,
        name
    );
}

#[allow(dead_code)]
fn assert_genesis_resouce_not_exist(
    chain_state: &ChainStateDB,
    module: &str,
    name: &str,
    type_params: Vec<TypeTag>,
) {
    let checkpoint = chain_state
        .get_state_value(&StateKey::AccessPath(AccessPath::new(
            genesis_address(),
            DataPath::Resource(StructTag {
                address: genesis_address(),
                module: Identifier::new(module).unwrap(),
                name: Identifier::new(name).unwrap(),
                type_params,
            }),
        )))
        .unwrap();
    assert!(
        checkpoint.is_none(),
        "expect genesis_account has no resource 0x1::{:?}::{:?}, but got it.",
        module,
        name
    );
}
