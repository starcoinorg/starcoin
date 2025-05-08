use crate::cli_state_trait::CliStateTrait;
use crate::{account, chain, contract, dev, helper, node, state, txpool};
use scmd::{CmdContext, CustomCommand};
use starcoin_account_provider::ProviderFactory;
use starcoin_config::{Connect, StarcoinOpt, G_APP_VERSION, G_CRATE_VERSION};
use starcoin_logger::prelude::{error, info};
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use std::time::Duration;

pub fn run<State: CliStateTrait, GlobalOpt>() -> anyhow::Result<()> {
    let logger_handle = starcoin_logger::init();
    let context = CmdContext::<State, GlobalOpt>::with_default_action(
        G_CRATE_VERSION,
        Some(G_APP_VERSION.as_str()),
        |opt| -> anyhow::Result<State> {
            info!("Starcoin opts: {}", opt);
            let connect = opt.connect().as_ref().unwrap_or(&Connect::IPC(None));
            let (client, node_handle) = match connect {
                Connect::IPC(ipc_file) => {
                    if let Some(ipc_file) = ipc_file {
                        info!("Try to connect node by ipc: {:?}", ipc_file);
                        let client = RpcClient::connect_ipc(ipc_file)?;
                        (client, None)
                    } else {
                        info!("Start starcoin node...");
                        let (node_handle, config) = starcoin_node::run_node_by_opt(opt)?;
                        match node_handle {
                            //first cli use local connect.
                            Some(node_handle) => {
                                info!("Connect by in process channel");
                                let rpc_service = node_handle.rpc_service()?;
                                let client = RpcClient::connect_local(rpc_service)?;
                                (client, Some(node_handle))
                            }
                            None => {
                                let ipc_file = config.rpc.get_ipc_file();
                                helper::wait_until_file_created(ipc_file.as_path())?;
                                info!(
                                    "Attach a new console by ipc: starcoin -c {} console",
                                    ipc_file.to_str().expect("invalid ipc file path.")
                                );
                                if let Some(ws_address) = config.rpc.get_ws_address() {
                                    info!(
                                        "Attach a new console by rpc: starcoin -c {} console",
                                        ws_address
                                    );
                                }
                                info!("Starcoin node started.");
                                info!("Try to connect node by ipc: {:?}", ipc_file);
                                let client = RpcClient::connect_ipc(ipc_file)?;
                                (client, None)
                            }
                        }
                    }
                }
                Connect::WebSocket(address) => {
                    info!("Try to connect node by websocket: {:?}", address);
                    let client = RpcClient::connect_websocket(address)?;
                    (client, None)
                }
            };

            let node_info = client.node_info()?;
            let client = Arc::new(client);
            let rpc_client = ProviderFactory::create_provider(
                client.clone(),
                node_info.net.chain_id(),
                &opt.account_provider(),
            )?;
            let state = State::new(
                node_info.net,
                client,
                opt.watch_timeout().map(Duration::from_secs),
                node_handle,
                rpc_client,
            );
            Ok(state)
        },
        |_, _, state| {
            let (_, client, handle) = state.into_inner();
            match Arc::try_unwrap(client) {
                Err(_) => {
                    error!("Can not close rpc client normal.");
                }
                Ok(client) => {
                    client.close();
                }
            }
            if let Some(handle) = handle {
                if let Err(e) = handle.join() {
                    error!("{:?}", e);
                }
            }
        },
    );
    let context = context.with_console_support(
        move |_app, _opt, state| {
            info!("Start console, disable stderr output.");
            logger_handle.disable_stderr();
            print_logo();
            (*scmd::G_DEFAULT_CONSOLE_CONFIG, Some(state.history_file()))
        },
        |_, _, state| {
            let (_, _, handle) = state.into_inner();
            if let Some(handle) = handle {
                if let Err(e) = handle.stop() {
                    error!("{:?}", e);
                }
            }
        },
    );
    add_command(context).exec()
}

pub fn add_command<State, GlobalOpt>(
    context: CmdContext<State, GlobalOpt>,
) -> CmdContext<State, GlobalOpt> {
    context
        .command(
            CustomCommand::with_name("account")
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

#[rustfmt::skip]
#[allow(clippy::print_literal)]
pub(crate) fn print_logo() {
    println!("{}{}{}","\x1b[34;1m",r#"
                                                      ::::::::███████::
                                                :::::███████████████████:
                                  :█████████:  :████████:::        :██████
                                  █████:█████:  █::                  █████:
                                 █████: :█████:                     :█████
                                :█████   :████:                    :█████:
                               :█████:    :████:                  :█████:
                              :█████:      █████:               :██████
               :███████████████████:       :███████████████:   ██████:
              :████████████████████         :████████████:   :█████:
              █████:                                      :██████:
             ::███████::                                :██████:
          :█::  ::████████:                          :███████:
        :████:::    :████████:                    ::██████:
      :█████::         ::█████:                  :█████::
    :██████:            :████:                   :█████:
   :█████:             :█████           :         :█████:
  :████:              :█████:        :█████::      :█████:
 :████:              :█████:     :█████████████:    :█████
:█████               █████: :██████████: :████████:  :████:
:████:               :████████████::        ::████████████:
 :█████::::::::::::██   ::███::                 :██████:
  :█████████████████::
      :::█████::::

         ██████╗████████╗  ███╗  ██████╗  █████╗  █████╗ ██╗███╗  ██╗
        ██╔════╝╚══██╔══╝ ██ ██╗ ╚════██╗██╔══██╗██╔══██╗██║████╗ ██║
        ╚█████╗    ██║   ██   ██║██████╔╝██║  ╚═╝██║  ██║██║██╔██╗██║
         ╚═══██╗   ██║   ██╔══██║██╔══██╗██║  ██╗██║  ██║██║██║╚████║
        ██████╔╝   ██║   ██║  ██║██║  ██║╚█████╔╝╚█████╔╝██║██║ ╚███║
        ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚════╝  ╚════╝ ╚═╝╚═╝  ╚══╝
"#,"\x1b[0m");
}
