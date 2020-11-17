// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use anyhow::{format_err, Result};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::transaction::RawUserTransaction;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::association_address;
use starcoin_vm_types::transaction::{SignedUserTransaction, TransactionPayload};
use starcoin_vm_types::{genesis_config::ChainNetwork, on_chain_config::DaoConfig};

pub fn sign_txn_with_account_by_rpc_client(
    cli_state: &CliState,
    addr: AccountAddress,
    max_gas_amount: u64,
    gas_price: u64,
    expiration_time: u64,
    payload: TransactionPayload,
) -> Result<SignedUserTransaction> {
    sign_txn_by_rpc_client(
        cli_state,
        max_gas_amount,
        gas_price,
        expiration_time,
        payload,
        Some(addr),
    )
}

pub fn _sign_txn_with_association_account_by_rpc_client(
    cli_state: &CliState,
    max_gas_amount: u64,
    gas_price: u64,
    expiration_time: u64,
    payload: TransactionPayload,
) -> Result<SignedUserTransaction> {
    sign_txn_by_rpc_client(
        cli_state,
        max_gas_amount,
        gas_price,
        expiration_time,
        payload,
        Some(association_address()),
    )
}

pub fn _sign_txn_with_default_account_by_rpc_client(
    cli_state: &CliState,
    max_gas_amount: u64,
    gas_price: u64,
    expiration_time: u64,
    payload: TransactionPayload,
) -> Result<SignedUserTransaction> {
    sign_txn_by_rpc_client(
        cli_state,
        max_gas_amount,
        gas_price,
        expiration_time,
        payload,
        None,
    )
}

fn sign_txn_by_rpc_client(
    cli_state: &CliState,
    max_gas_amount: u64,
    gas_price: u64,
    expiration_time: u64,
    payload: TransactionPayload,
    account_address: Option<AccountAddress>,
) -> Result<SignedUserTransaction> {
    let account = cli_state.get_account_or_default(account_address)?;
    let client = cli_state.client();
    let node_info = client.node_info()?;
    let chain_state_reader = RemoteStateReader::new(client);
    let account_state_reader = AccountStateReader::new(&chain_state_reader);
    let account_resource = account_state_reader
        .get_account_resource(account.address())?
        .ok_or_else(|| format_err!("account {:?} must exist on chain.", account.address()))?;
    let net = ChainNetwork::new_builtin(
        *cli_state
            .net()
            .as_builtin()
            .ok_or_else(|| format_err!("Only support builtin network"))?,
    );
    let expiration_time = expiration_time + node_info.now_seconds;
    let raw_txn = RawUserTransaction::new(
        account.address,
        account_resource.sequence_number(),
        payload,
        max_gas_amount,
        gas_price,
        expiration_time,
        net.chain_id(),
    );

    client.account_sign_txn(raw_txn)
}

pub fn get_dao_config(cli_state: &CliState) -> Result<DaoConfig> {
    let client = cli_state.client();
    let chain_state_reader = RemoteStateReader::new(client);
    let account_state_reader = AccountStateReader::new(&chain_state_reader);
    Ok(account_state_reader
        .get_on_chain_config::<DaoConfig>()?
        .ok_or_else(|| format_err!("DaoConfig not exist on chain."))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_config::NodeConfig;
    use starcoin_crypto::hash::PlainCryptoHash;
    use starcoin_logger::prelude::*;
    use starcoin_node::NodeHandle;
    use starcoin_rpc_api::types::{AnnotatedMoveValue, ContractCall};
    use starcoin_rpc_client::RpcClient;
    use starcoin_transaction_builder::{
        build_module_upgrade_plan, build_module_upgrade_proposal, build_module_upgrade_queue,
    };
    use starcoin_types::transaction::{parse_transaction_argument, Script, TransactionArgument};
    use starcoin_vm_types::{
        account_config::{association_address, genesis_address, AccountResource},
        genesis_config::StdlibVersion,
        transaction::Package,
        vm_status::KeptVMStatus,
    };
    use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
    use std::sync::Arc;
    use std::{thread::sleep, time::Duration};
    use stdlib::transaction_scripts::{compiled_transaction_script, StdlibScript};
    use test_helper::executor::compile_module_with_address;
    use test_helper::run_node_by_config;

    fn get_account_resource(
        cli_state: &CliState,
        addr: AccountAddress,
    ) -> Result<(AccountResource, u128)> {
        let chain_state_reader = RemoteStateReader::new(cli_state.client());
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader
            .get_account_resource(&addr)?
            .ok_or_else(|| format_err!("address address {} must exist", addr))?;

        let balance = account_state_reader
            .get_balance(&addr)?
            .ok_or_else(|| format_err!("address address {} balance must exist", addr))?;

        Ok((account_resource, balance))
    }

    fn create_default_account(
        cli_state: &CliState,
        config: &Arc<NodeConfig>,
        node_handle: &NodeHandle,
    ) -> u128 {
        let default_account = cli_state.default_account().unwrap();
        // unlock default account
        cli_state
            .client()
            .account_unlock(
                default_account.address,
                "".to_string(),
                Duration::from_secs(100),
            )
            .unwrap();
        let (association_account_resource, association_balance) =
            get_account_resource(&cli_state, association_address()).unwrap();
        let transfer_amount = association_balance * 90 / 100;
        info!(
            "association_balance : {}, {}",
            association_balance,
            association_balance * 90 / 100
        );
        let seq_num = association_account_resource.sequence_number();
        let transfer_raw_txn = starcoin_executor::build_transfer_txn(
            association_address(),
            default_account.address,
            Some(default_account.public_key.auth_key()),
            seq_num,
            transfer_amount,
            1,
            1_000_000,
            3_000 + config.net().time_service().now_secs(),
            cli_state.net().chain_id(),
        );
        let transfer_txn = cli_state
            .client()
            .account_sign_txn(transfer_raw_txn)
            .unwrap();
        let transfer_txn_id = transfer_txn.crypto_hash();
        cli_state
            .client()
            .submit_transaction(transfer_txn.clone())
            .unwrap()
            .unwrap();
        sleep(Duration::from_millis(500));
        let block = node_handle.generate_block().unwrap();
        assert!(block.transactions().contains(&transfer_txn));
        let transfer_txn_info = cli_state
            .client()
            .chain_get_transaction_info(transfer_txn_id)
            .unwrap()
            .unwrap();
        assert_eq!(transfer_txn_info.status(), &KeptVMStatus::Executed);
        transfer_amount
    }

    #[stest::test(timeout = 300)]
    fn test_upgrade_module() {
        let mut node_config = NodeConfig::random_for_test();
        node_config.network.disable_seed = true;
        let config = Arc::new(node_config);
        let node_handle = run_node_by_config(config.clone()).unwrap();
        let rpc_service = node_handle.rpc_service().unwrap();
        let rpc_client = RpcClient::connect_local(rpc_service).unwrap();
        let node_info = rpc_client.node_info().unwrap();
        let cli_state = CliState::new(node_info.net, Arc::new(rpc_client), None, None, None);
        cli_state
            .client()
            .account_unlock(
                association_address(),
                "".to_string(),
                Duration::from_secs(100),
            )
            .unwrap();

        // 1. proposal
        let test_upgrade_module_source = r#"
        module TestModule {
            public fun is_test(): bool {
                true
            }
        }
        "#;
        let test_upgrade_module =
            compile_module_with_address(genesis_address(), test_upgrade_module_source);
        let test_upgrade_module_package = Package::new_with_module(test_upgrade_module).unwrap();

        let dao_config = config.net().genesis_config().dao_config;
        let (module_upgrade_proposal, _) = build_module_upgrade_proposal(
            &test_upgrade_module_package,
            2,
            dao_config.min_action_delay,
        );

        let proposal_txn = _sign_txn_with_association_account_by_rpc_client(
            &cli_state,
            1_000_000,
            1,
            3000,
            TransactionPayload::Script(module_upgrade_proposal),
        )
        .unwrap();

        let proposal_txn_id = proposal_txn.crypto_hash();
        cli_state
            .client()
            .submit_transaction(proposal_txn.clone())
            .unwrap()
            .unwrap();
        sleep(Duration::from_millis(500));
        let block = node_handle.generate_block().unwrap();
        assert!(block.transactions().contains(&proposal_txn));
        let proposal_txn_info = cli_state
            .client()
            .chain_get_transaction_info(proposal_txn_id)
            .unwrap()
            .unwrap();
        info!("txn status : {:?}", proposal_txn_info);
        assert_eq!(proposal_txn_info.status(), &KeptVMStatus::Executed);

        // 2. transfer
        cli_state
            .client()
            .sleep(dao_config.voting_period / 2)
            .unwrap();
        let default_account = cli_state.default_account().unwrap();
        // unlock default account
        let transfer_amount = create_default_account(&cli_state, &config, &node_handle);

        // 3. vote
        let proposal_id = 0;
        let vote_code =
            compiled_transaction_script(StdlibVersion::Latest, StdlibScript::CastVote).into_vec();
        let mut type_tags: Vec<TypeTag> = Vec::new();
        let stc = parse_type_tag("0x1::STC::STC").unwrap();
        let module = parse_type_tag("0x1::UpgradeModuleDaoProposal::UpgradeModule").unwrap();
        type_tags.push(stc);
        type_tags.push(module);
        let mut args: Vec<TransactionArgument> = Vec::new();
        let arg_1 = parse_transaction_argument("0x0000000000000000000000000a550c18").unwrap();
        let arg_2 = parse_transaction_argument(&format!("{}", proposal_id)).unwrap();
        let arg_3 = parse_transaction_argument("true").unwrap();
        let arg_4 =
            parse_transaction_argument(&format!("{}u128", transfer_amount * 90 / 100)).unwrap();
        args.push(arg_1);
        args.push(arg_2);
        args.push(arg_3);
        args.push(arg_4);
        let vote_raw_txn = RawUserTransaction::new_script(
            default_account.address,
            0,
            Script::new(vote_code, type_tags, args),
            1_000_000,
            1,
            3_000 + config.net().time_service().now_secs(),
            cli_state.net().chain_id(),
        );
        let vote_txn = cli_state.client().account_sign_txn(vote_raw_txn).unwrap();
        let vote_txn_id = vote_txn.crypto_hash();
        cli_state
            .client()
            .submit_transaction(vote_txn.clone())
            .unwrap()
            .unwrap();
        sleep(Duration::from_millis(500));
        let block = node_handle.generate_block().unwrap();
        assert!(block.transactions().contains(&vote_txn));
        let vote_txn_info = cli_state
            .client()
            .chain_get_transaction_info(vote_txn_id)
            .unwrap()
            .unwrap();
        assert_eq!(vote_txn_info.status(), &KeptVMStatus::Executed);

        // 4. sleep
        cli_state.client().sleep(dao_config.voting_period).unwrap();
        sleep(Duration::from_millis(500));
        node_handle.generate_block().unwrap();

        // 5. queue
        let module_upgrade_queue = build_module_upgrade_queue(association_address(), proposal_id);
        let queue_txn = sign_txn_with_account_by_rpc_client(
            &cli_state,
            default_account.address,
            1_000_000,
            1,
            3_000,
            TransactionPayload::Script(module_upgrade_queue),
        )
        .unwrap();
        let queue_txn_id = queue_txn.crypto_hash();
        cli_state
            .client()
            .submit_transaction(queue_txn.clone())
            .unwrap()
            .unwrap();
        sleep(Duration::from_millis(500));
        let block = node_handle.generate_block().unwrap();
        assert!(block.transactions().contains(&queue_txn));
        let queue_txn_info = cli_state
            .client()
            .chain_get_transaction_info(queue_txn_id)
            .unwrap()
            .unwrap();
        info!("queue txn info : {:?}", queue_txn_info);
        assert_eq!(queue_txn_info.status(), &KeptVMStatus::Executed);

        // 6. sleep
        cli_state.client().sleep(dao_config.voting_period).unwrap();
        sleep(Duration::from_millis(500));
        node_handle.generate_block().unwrap();

        // 7. plan
        let module_upgrade_plan = build_module_upgrade_plan(association_address(), proposal_id);
        let plan_txn = sign_txn_with_account_by_rpc_client(
            &cli_state,
            default_account.address,
            1_000_000,
            1,
            3_000,
            TransactionPayload::Script(module_upgrade_plan),
        )
        .unwrap();
        let plan_txn_id = plan_txn.crypto_hash();
        cli_state
            .client()
            .submit_transaction(plan_txn.clone())
            .unwrap()
            .unwrap();
        sleep(Duration::from_millis(500));
        let block = node_handle.generate_block().unwrap();
        assert!(block.transactions().contains(&plan_txn));
        let plan_txn_info = cli_state
            .client()
            .chain_get_transaction_info(plan_txn_id)
            .unwrap()
            .unwrap();
        assert_eq!(plan_txn_info.status(), &KeptVMStatus::Executed);

        // 8. exe package
        let package_txn = _sign_txn_with_association_account_by_rpc_client(
            &cli_state,
            1_000_000,
            1,
            3_000,
            TransactionPayload::Package(test_upgrade_module_package),
        )
        .unwrap();
        let package_txn_id = package_txn.crypto_hash();
        cli_state
            .client()
            .submit_transaction(package_txn.clone())
            .unwrap()
            .unwrap();
        sleep(Duration::from_millis(500));
        let block = node_handle.generate_block().unwrap();
        assert!(block.transactions().contains(&package_txn));
        let package_txn_info = cli_state
            .client()
            .chain_get_transaction_info(package_txn_id)
            .unwrap()
            .unwrap();
        assert_eq!(package_txn_info.status(), &KeptVMStatus::Executed);

        // 9. verify
        let call = ContractCall {
            module_address: genesis_address(),
            module_name: "TestModule".to_string(),
            func: "is_test".to_string(),
            type_args: Vec::new(),
            args: Vec::new(),
        };
        let result = cli_state.client().contract_call(call).unwrap();
        assert!(!result.is_empty());
        info!("result: {:?}", result);
        if let AnnotatedMoveValue::Bool(flag) = result.get(0).unwrap() {
            assert!(flag);
        } else {
            unreachable!("result err.");
        }

        node_handle.stop().unwrap();
    }

    #[stest::test(timeout = 300)]
    fn test_only_new_module() {
        let mut node_config = NodeConfig::random_for_test();
        node_config.network.disable_seed = true;
        let config = Arc::new(node_config);
        let node_handle = run_node_by_config(config.clone()).unwrap();
        let rpc_service = node_handle.rpc_service().unwrap();
        let rpc_client = RpcClient::connect_local(rpc_service).unwrap();
        let node_info = rpc_client.node_info().unwrap();
        let cli_state = CliState::new(node_info.net, Arc::new(rpc_client), None, None, None);
        cli_state
            .client()
            .account_unlock(
                association_address(),
                "".to_string(),
                Duration::from_secs(100),
            )
            .unwrap();

        // 1. create account
        let default_account = cli_state.default_account().unwrap();
        let _ = create_default_account(&cli_state, &config, &node_handle);

        // 2. set only_new_module strategy
        let only_new_module_strategy = compiled_transaction_script(
            StdlibVersion::Latest,
            StdlibScript::UpdateModuleUpgradeStrategy,
        )
        .into_vec();
        let mut args: Vec<TransactionArgument> = Vec::new();
        let arg = parse_transaction_argument(&format!("{}u8", 2)).unwrap();
        args.push(arg);
        let only_new_module_strategy_raw_txn = RawUserTransaction::new_script(
            default_account.address,
            0,
            Script::new(only_new_module_strategy, Vec::new(), args),
            1_000_000,
            1,
            3_000 + config.net().time_service().now_secs(),
            cli_state.net().chain_id(),
        );
        let only_new_module_strategy_txn = cli_state
            .client()
            .account_sign_txn(only_new_module_strategy_raw_txn)
            .unwrap();
        let only_new_module_strategy_txn_id = only_new_module_strategy_txn.crypto_hash();
        cli_state
            .client()
            .submit_transaction(only_new_module_strategy_txn.clone())
            .unwrap()
            .unwrap();
        sleep(Duration::from_millis(500));
        let block = node_handle.generate_block().unwrap();
        assert!(block.transactions().contains(&only_new_module_strategy_txn));
        let only_new_module_strategy_txn_info = cli_state
            .client()
            .chain_get_transaction_info(only_new_module_strategy_txn_id)
            .unwrap()
            .unwrap();
        assert_eq!(
            only_new_module_strategy_txn_info.status(),
            &KeptVMStatus::Executed
        );

        // 3. apply new module
        let test_upgrade_module_source_1 = r#"
        module TestModule {
            public fun is_test(): bool {
                true
            }
        }
        "#;
        let test_upgrade_module_1 =
            compile_module_with_address(default_account.address, test_upgrade_module_source_1);
        let test_upgrade_module_package_1 =
            Package::new_with_module(test_upgrade_module_1).unwrap();
        let package_txn_1 = _sign_txn_with_association_account_by_rpc_client(
            &cli_state,
            1_000_000,
            1,
            3_000,
            TransactionPayload::Package(test_upgrade_module_package_1),
        )
        .unwrap();
        let package_txn_id_1 = package_txn_1.crypto_hash();
        cli_state
            .client()
            .submit_transaction(package_txn_1.clone())
            .unwrap()
            .unwrap();
        sleep(Duration::from_millis(500));
        let block = node_handle.generate_block().unwrap();
        assert!(block.transactions().contains(&package_txn_1));
        let package_txn_info_1 = cli_state
            .client()
            .chain_get_transaction_info(package_txn_id_1)
            .unwrap()
            .unwrap();
        assert_eq!(package_txn_info_1.status(), &KeptVMStatus::Executed);

        // 4. 更新module
        let test_upgrade_module_source_2 = r#"
        module TestModule {
            public fun is_test(): bool {
                true
            }

            public fun update_test(): bool {
                true
            }
        }
        "#;
        let test_upgrade_module_2 =
            compile_module_with_address(default_account.address, test_upgrade_module_source_2);
        let test_upgrade_module_package_2 =
            Package::new_with_module(test_upgrade_module_2).unwrap();
        let package_txn_2 = _sign_txn_with_association_account_by_rpc_client(
            &cli_state,
            1_000_000,
            1,
            3_000,
            TransactionPayload::Package(test_upgrade_module_package_2),
        )
        .unwrap();
        let result = cli_state
            .client()
            .submit_transaction(package_txn_2)
            .unwrap();
        assert!(result.is_err());
        info!("error : {:?}", result);

        node_handle.stop().unwrap();
    }
}
