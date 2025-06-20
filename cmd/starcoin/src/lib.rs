// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod account;
pub mod chain;
pub mod cli_state;
mod cli_state_vm2;
pub mod contract;
pub mod dev;
pub mod helper;
pub mod mutlisig_transaction;
pub mod mutlisig_transaction_vm2;
pub mod node;
pub mod state;
pub mod subcommand_vm2;
pub mod txpool;
pub mod view;
pub mod view_vm2;

pub use cli_state::CliState;
use scmd::{CmdContext, CustomCommand};
pub use starcoin_config::StarcoinOpt;
pub use starcoin_node::crash_handler;

use subcommand_vm2::{account as account2, contract as contract2, dev as dev2};

pub fn add_command(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context
        .command(
            CustomCommand::with_name("account")
                .subcommand(account2::CreateCommand)
                .subcommand(account2::ShowCommand)
                .subcommand(account2::TransferCommand)
                .subcommand(account2::AcceptTokenCommand)
                .subcommand(account2::ListCommand)
                .subcommand(account2::ImportMultisigCommand)
                .subcommand(account2::ChangePasswordCmd)
                .subcommand(account2::DefaultCommand)
                .subcommand(account2::remove_cmd::RemoveCommand)
                .subcommand(account2::LockCommand)
                .subcommand(account2::UnlockCommand)
                .subcommand(account2::ExportCommand)
                .subcommand(account2::ImportCommand)
                .subcommand(account2::ImportReadonlyCommand)
                .subcommand(account2::ExecuteScriptFunctionCmd)
                .subcommand(account2::ExecuteScriptCommand)
                .subcommand(account2::GenerateMultisigTxnCommand)
                .subcommand(account2::SubmitSignedTxnCommand)
                .subcommand(account2::SignMessageCmd)
                .subcommand(account2::VerifySignMessageCmd)
                .subcommand(account::DeriveAddressCommand)
                .subcommand(account::receipt_identifier_cmd::ReceiptIdentifierCommand)
                .subcommand(account::generate_keypair::GenerateKeypairCommand)
                .subcommand(account2::RotateAuthenticationKeyCommand), // .subcommand(account::nft_cmd::NFTCommand), // TODO(BobOng): [multi-vm] to handle nft related command for vm2
        )
        .command(
            CustomCommand::with_name("state")
                .subcommand(state::ListCmd)
                .subcommand(state::GetCommand)
                .subcommand(state::GetProofCommand)
                .subcommand(state::GetRootCommand),
        )
        .command(
            CustomCommand::with_name("node")
                .subcommand(node::InfoCommand)
                .subcommand(node::PeersCommand)
                .subcommand(node::MetricsCommand)
                .subcommand(node::manager::NodeManagerCommand)
                .subcommand(
                    CustomCommand::with_name("service")
                        .subcommand(node::service::ListCommand)
                        .subcommand(node::service::StartCommand)
                        .subcommand(node::service::CheckCommand)
                        .subcommand(node::service::StopCommand), //TODO support shutdown by command
                                                                 //.subcommand(node::service::ShutdownSystemCommand),
                )
                .subcommand(
                    CustomCommand::with_name("sync")
                        .subcommand(node::sync::StartCommand)
                        .subcommand(node::sync::StatusCommand)
                        .subcommand(node::sync::ProgressCommand)
                        .subcommand(node::sync::CancelCommand)
                        .subcommand(node::sync::PeerScoreCommand),
                )
                .subcommand(
                    CustomCommand::with_name("network")
                        .subcommand(node::network::StateCommand)
                        .subcommand(node::network::KnownPeersCommand)
                        .subcommand(node::network::GetAddressCommand)
                        .subcommand(node::network::AddPeerCommand)
                        .subcommand(node::network::CallPeerCommand)
                        .subcommand(node::network::SetPeerReputation)
                        .subcommand(node::network::BanPeerCommand),
                ),
        )
        .command(
            CustomCommand::with_name("chain")
                .subcommand(chain::InfoCommand)
                .subcommand(chain::GetBlockCommand)
                .subcommand(chain::ListBlockCommand)
                .subcommand(chain::GetTransactionCommand)
                .subcommand(chain::GetTxnInfosCommand)
                .subcommand(chain::GetTransactionInfoCommand)
                .subcommand(chain::GetEventsCommand)
                .subcommand(chain::EpochInfoCommand)
                .subcommand(chain::GetTransactionInfoListCommand)
                .subcommand(chain::get_txn_proof_cmd::GetTransactionProofCommand)
                .subcommand(chain::GetBlockInfoCommand),
        )
        .command(
            CustomCommand::with_name("txpool")
                .subcommand(txpool::PendingTxnCommand)
                .subcommand(txpool::PendingTxnsCommand)
                .subcommand(txpool::TxPoolStatusCommand),
        )
        .command(
            CustomCommand::with_name("dev")
                .subcommand(dev2::GetCoinCommand)
                .subcommand(dev2::MoveExplain)
                .subcommand(dev2::CompileCommand)
                .subcommand(dev2::DeployCommand)
                .subcommand(dev2::PackageCmd)
                .subcommand(dev2::CallContractCommand)
                .subcommand(dev2::ResolveCommand)
                .subcommand(dev2::CallApiCommand)
                .subcommand(
                    CustomCommand::with_name("subscribe")
                        .with_about("Subscribe the chain events")
                        .subcommand(dev::SubscribeNewMintBlockCommand)
                        .subcommand(dev::SubscribeBlockCommand)
                        .subcommand(dev::SubscribeEventCommand)
                        .subcommand(dev::SubscribeNewTxnCommand),
                )
                .subcommand(
                    CustomCommand::with_name("log")
                        .with_about("Set node's log level and pattern.")
                        .subcommand(dev::log_cmd::LogLevelCommand)
                        .subcommand(dev::log_cmd::LogPatternCommand),
                )
                .subcommand(dev::panic_cmd::PanicCommand)
                .subcommand(dev::sleep_cmd::SleepCommand)
                .subcommand(dev2::gen_block_cmd::GenBlockCommand)
                .subcommand(dev2::SetConcurrencyLevelCommand)
                .subcommand(dev2::GetConcurrencyLevelCommand)
                .subcommand(dev::SetLoggerBalanceAmoutCommand)
                .subcommand(dev::GetLoggerBalanceAmountCommand),
        )
        .command(CustomCommand::with_name("contract").subcommand(contract2::GetContractDataCommand))
}

/// The legacy command will handle vm1 related commands
pub fn add_command_legacy(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context
        .command(
            CustomCommand::with_name("account1")
                .subcommand(account::CreateCommand)
                .subcommand(account::ShowCommand)
                .subcommand(account::TransferCommand)
                .subcommand(account::AcceptTokenCommand)
                .subcommand(account::ListCommand)
                .subcommand(account::import_multisig_cmd::ImportMultisigCommand)
                .subcommand(account::ChangePasswordCmd)
                .subcommand(account::DefaultCommand)
                .subcommand(account::remove_cmd::RemoveCommand)
                .subcommand(account::LockCommand)
                .subcommand(account::UnlockCommand)
                .subcommand(account::ExportCommand)
                .subcommand(account::ImportCommand)
                .subcommand(account::import_readonly_cmd::ImportReadonlyCommand)
                .subcommand(account::ExecuteScriptFunctionCmd)
                .subcommand(account::ExecuteScriptCommand)
                .subcommand(account::sign_multisig_txn_cmd::GenerateMultisigTxnCommand)
                .subcommand(account::submit_txn_cmd::SubmitSignedTxnCommand)
                .subcommand(account::SignMessageCmd)
                .subcommand(account::VerifySignMessageCmd)
                .subcommand(account::DeriveAddressCommand)
                .subcommand(account::receipt_identifier_cmd::ReceiptIdentifierCommand)
                .subcommand(account::generate_keypair::GenerateKeypairCommand)
                .subcommand(account::rotate_auth_key_cmd::RotateAuthenticationKeyCommand)
                .subcommand(account::nft_cmd::NFTCommand),
        )
        .command(
            CustomCommand::with_name("dev1")
                .subcommand(dev::GetCoinCommand)
                .subcommand(dev::move_explain::MoveExplain)
                .subcommand(dev::CompileCommand)
                .subcommand(dev::DeployCommand)
                .subcommand(dev::UpgradeModuleProposalCommand)
                .subcommand(dev::UpgradeModulePlanCommand)
                .subcommand(dev::UpgradeModuleQueueCommand)
                .subcommand(dev::UpgradeModuleExeCommand)
                .subcommand(dev::UpgradeVMConfigProposalCommand)
                .subcommand(dev::PackageCmd)
                .subcommand(dev::CallContractCommand)
                .subcommand(dev::resolve_cmd::ResolveCommand)
                .subcommand(dev::call_api_cmd::CallApiCommand)
                .subcommand(
                    CustomCommand::with_name("subscribe")
                        .with_about("Subscribe the chain events")
                        .subcommand(dev::SubscribeNewMintBlockCommand)
                        .subcommand(dev::SubscribeBlockCommand)
                        .subcommand(dev::SubscribeEventCommand)
                        .subcommand(dev::SubscribeNewTxnCommand),
                )
                .subcommand(
                    CustomCommand::with_name("log")
                        .with_about("Set node's log level and pattern.")
                        .subcommand(dev::log_cmd::LogLevelCommand)
                        .subcommand(dev::log_cmd::LogPatternCommand),
                )
                .subcommand(dev::panic_cmd::PanicCommand)
                .subcommand(dev::sleep_cmd::SleepCommand)
                .subcommand(dev::gen_block_cmd::GenBlockCommand)
                .subcommand(dev::SetConcurrencyLevelCommand)
                .subcommand(dev::GetConcurrencyLevelCommand)
                .subcommand(dev::SetLoggerBalanceAmoutCommand)
                .subcommand(dev::GetLoggerBalanceAmountCommand),
        )
        .command(CustomCommand::with_name("contract").subcommand(contract::GetContractDataCommand))
}
