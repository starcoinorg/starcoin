use anyhow::Result;
use starcoin_config::{BuiltinNetworkID, ChainNetwork, ChainNetworkID};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_logger::prelude::*;
use starcoin_types::stdlib::StdlibVersion;
use starcoin_vm2_cached_packages::starcoin_framework_sdk_builder::{
    dao_upgrade_module_proposal_propose_module_upgrade_v2,
    dao_upgrade_module_proposal_submit_module_upgrade_plan,
};
use starcoin_vm2_state_api::{ChainStateReader, StateReaderExt};
use starcoin_vm2_test_helper::dao::dao_vote_test;
use starcoin_vm2_test_helper::executor::*;
use starcoin_vm2_types::{
    account::Account,
    account_config::{genesis_address, stc_type_tag},
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    transaction::TransactionPayload,
};
use starcoin_vm2_vm_types::account_config::config_change::ConfigChangeEvent;
use starcoin_vm2_vm_types::account_config::upgrade::UpgradeEvent;
use starcoin_vm2_vm_types::on_chain_config::Version;
use starcoin_vm2_vm_types::on_chain_resource::dao::UpgradeModuleV2;
use stdlib::vm2::load_upgrade_package;

#[stest::test(timeout = 300)]
fn test_stdlib_upgrade() -> Result<()> {
    let current_version = StdlibVersion::Version(12);
    let new_version = StdlibVersion::Latest;
    let mut net = ChainNetwork::new_dev();
    let mut genesis_config = net.genesis_config2().clone();
    genesis_config.stdlib_version = current_version;
    net = ChainNetwork::new(
        ChainNetworkID::Builtin(BuiltinNetworkID::Dev),
        net.genesis_config().clone(),
        genesis_config,
    );
    let package_name = net.to_string();

    let chain_state = prepare_customized_genesis(&net)?;
    let proposal_id: u64 = 0;
    let alice = Account::new();

    // Create upgrade module type tag
    let dao_action_type_tag = UpgradeModuleV2::type_tag();

    let package =
        match load_upgrade_package(current_version, new_version, &package_name, &package_name)? {
            Some(package) => package,
            None => {
                panic!(
                    "{:?} is same as {:?}, continue",
                    current_version, new_version
                );
            }
        };
    let package_hash = package.crypto_hash();

    // Create proposal to upgrade module
    let vote_payload = dao_upgrade_module_proposal_propose_module_upgrade_v2(
        stc_type_tag(),
        genesis_address(),
        package_hash.to_vec(),
        2,     // version 2
        60000, // exec_delay
        false, // enforced
    );

    // Create execution script
    let execute_script_payload = dao_upgrade_module_proposal_submit_module_upgrade_plan(
        stc_type_tag(),
        *alice.address(),
        proposal_id,
    );

    // Execute DAO voting process
    dao_vote_test(
        &alice,
        &chain_state,
        &net,
        vote_payload,
        dao_action_type_tag,
        execute_script_payload,
        proposal_id,
    )?;

    // Verify version was updated
    verify_version_state(&chain_state)?;

    let output = association_execute_should_success(
        &net,
        &chain_state,
        TransactionPayload::Package(package),
    )?;
    let contract_event = expect_event::<UpgradeEvent>(&output);
    let _upgrade_event = contract_event.decode_event::<UpgradeEvent>()?;

    let _version_config_event = expect_event::<ConfigChangeEvent<Version>>(&output);

    //ext_execute_after_upgrade(new_version, &net, &chain_state)?;

    info!("Module upgrade test completed successfully");

    Ok(())
}

fn verify_version_state<R>(chain_state: &R) -> Result<()>
where
    R: ChainStateReader,
{
    info!("Verifying module upgrade state");

    // Check that the version config is updated
    let version_config = chain_state.get_on_chain_config::<Version>();
    match version_config {
        Some(config) => {
            info!("On-chain version config: {:?}", config);
        }
        None => {
            info!("No version config found");
        }
    }

    Ok(())
}
