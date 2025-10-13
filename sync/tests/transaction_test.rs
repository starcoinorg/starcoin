use std::{env::temp_dir, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use network_api::PeerId;
use starcoin_logger::{prelude::{info, LevelFilter}, LoggerHandle};
use starcoin_types::{genesis_config::ChainId, multi_transaction::MultiSignedUserTransaction};
use starcoin_vm2_account_api::message::{AccountRequest, AccountResponse};
use starcoin_config::{genesis_config::CustomNetworkID, BaseConfig, ChainNetworkID, NodeConfig, StarcoinOpt, G_HALLEY_CONFIG};
use anyhow::{bail, Ok, Result, format_err};
use starcoin_network::{PeerTransactionsMessage, TransactionsMessage};
use starcoin_service_registry::RegistryAsyncService;
use starcoin_transaction_builder::vm2::build_batch_transfer_txn as build_batch_transfer_txn2;
use test_helper::run_node_by_config;
use starcoin_vm2_account_service::{
    AccountService as AccountService2, AccountStorage as AccountStorage2,
};



#[test]
fn test_transaction_in_custom_network() -> Result<()> {
      let mut opt = StarcoinOpt::default(); 
      opt.net = Some(ChainNetworkID::Custom(CustomNetworkID::new("my_chain".to_string(), ChainId::new(121))));
      let path = temp_dir();
      opt.base_data_dir = Some(path.clone());
      opt.genesis_config = Some("halley".to_string());
      let _ = BaseConfig::load_with_opt(&opt)?;

      let mut global_opt = StarcoinOpt::default();
      global_opt.net = Some(ChainNetworkID::Custom(CustomNetworkID::new("my_chain".to_string(), ChainId::new(121))));
      global_opt.base_data_dir = Some(path.clone());
      opt.genesis_config = Some(path.join("my_chain/genesis_config2.json").to_str().unwrap().to_string());
      let node_config = Arc::new(NodeConfig::load_with_opt(&global_opt)?);

      // let node_config = Arc::new(NodeConfig::random_for_test());
      let node = run_node_by_config(node_config.clone()).unwrap();
      let registry = node.registry();

      let genesis = node.genesis();

      // to mint blocks to get token 
      for _ in 0..10 {
          node.generate_block()?;
      }

      let fut = async move {
          let log_handler = registry.get_shared::<Arc<LoggerHandle>>().await?;
          log_handler.update_level(LevelFilter::Info);
          let txpool = registry.get_shared::<starcoin_txpool::TxPoolService>().await?;
          let account_service = registry.service_ref::<AccountService2>().await?;
          let default_account= match account_service.send(AccountRequest::GetDefaultAccount()).await?? {
              AccountResponse::AccountInfoOption(account) => account.ok_or_else(|| format_err!("default account not exist"))?,
              _ => bail!("Unexpect response type.")
          };
          let receiver= match account_service.send(AccountRequest::CreateAccount("".to_string())).await?? {
              AccountResponse::AccountInfo(account) => *account,
              _ => bail!("Unexpect response type.")
          };
          let config = registry.get_shared::<Arc<NodeConfig>>().await?;
          let expire_time = config.net().time_service().now_secs() + 3600;
          let transaction = build_batch_transfer_txn2(
            default_account.address, 
            vec![receiver.address], 
            0, 
            1,
            1,
            40_000_000,
            expire_time,
            genesis.block().header().chain_id().id().into(),
          );
          match account_service.send(AccountRequest::UnlockAccount(default_account.address, "".to_string(), std::time::Duration::from_secs(100))).await?? {
              AccountResponse::AccountInfo(_) => (),
              _ => bail!("Unexpect response type.")
          }
          let signed_transaction= match account_service.send(AccountRequest::SignTxn {
              txn: Box::new(transaction),
              signer: default_account.address
          }).await?? {
              AccountResponse::SignedTxn(signed_transction) => *signed_transction,
              _ => bail!("Unexpect response type.")
          };
          info!("jacktest: transaction in halley signed: id: {}", signed_transaction.id());
          // let message = PeerTransactionsMessage::new(PeerId::random(), TransactionsMessage::new(vec![MultiSignedUserTransaction::VM2(signed_transaction)]));
          txpool.inner.import_txns(vec![MultiSignedUserTransaction::VM2(signed_transaction)], false, None)?;
          let transactions = txpool.inner.get_pending(100, 0)?;
          info!("jacktest: transaction in halley pending: {}", transactions.len());
          for txn in transactions {
              match txn.signed() {
                  MultiSignedUserTransaction::VM1(signed_user_transaction) => info!("jacktest: transaction1 in halley pending: id: {}", signed_user_transaction.id()),
                  MultiSignedUserTransaction::VM2(signed_user_transaction) => info!("jacktest: transaction2 in halley pending: id: {}", signed_user_transaction.id()),
              }
          }
          Ok(())
      };
      tokio::runtime::Runtime::new().unwrap().block_on(fut)?;
      std::thread::sleep(std::time::Duration::from_secs(10));
      let executed_block = node.generate_block()?;
      info!("jacktest: transaction in halley executed block: {}", executed_block.body.transactions2.len());
      Ok(())
}