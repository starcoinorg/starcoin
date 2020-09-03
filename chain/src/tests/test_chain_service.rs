// use crate::{BlockChain, ChainActor, ChainActorRef, ChainAsyncService};
// use anyhow::Result;
// use bus::BusActor;
// use consensus::Consensus;
// use crypto::{ed25519::Ed25519PrivateKey, hash::PlainCryptoHash, Genesis, PrivateKey};
// use logger::prelude::*;
// use starcoin_account_api::AccountInfo;
// use starcoin_config::NodeConfig;
// use starcoin_genesis::Genesis as StarcoinGenesis;
// use starcoin_traits::{ChainReader, ChainWriter};
// use starcoin_types::account_address;
// use starcoin_types::transaction::authenticator::AuthenticationKey;
// use std::sync::Arc;
// use storage::Storage;
// use txpool::TxPool;
//
// async fn gen_master_chain(times: u64) -> (ChainActorRef, Arc<NodeConfig>, Arc<Storage>) {
//     let node_config = NodeConfig::random_for_test();
//     let node_config = Arc::new(node_config);
//
//     let (storage, startup_info, _) = StarcoinGenesis::init_storage_for_test(node_config.net())
//         .expect("init storage by genesis fail.");
//
//     let bus = BusActor::launch();
//     let txpool_service = {
//         let best_block_id = *startup_info.get_master();
//         TxPool::start(
//             node_config.clone(),
//             storage.clone(),
//             best_block_id,
//             bus.clone(),
//         )
//         .get_service()
//     };
//     let chain = ChainActor::launch(
//         node_config.clone(),
//         startup_info.clone(),
//         storage.clone(),
//         txpool_service,
//         bus.clone(),
//     )
//     .unwrap();
//     let miner_account = AccountInfo::random();
//     let consensus_strategy = node_config.net().consensus();
//     if times > 0 {
//         for _i in 0..times {
//             let startup_info = chain.clone().master_startup_info().await.unwrap();
//             let block_chain = BlockChain::new(
//                 node_config.net().consensus(),
//                 startup_info.master,
//                 storage.clone(),
//             )
//             .unwrap();
//             let (block_template, _) = block_chain
//                 .create_block_template(
//                     *miner_account.address(),
//                     Some(miner_account.get_auth_key().prefix().to_vec()),
//                     None,
//                     Vec::new(),
//                     vec![],
//                     None,
//                 )
//                 .unwrap();
//             let block = consensus_strategy
//                 .create_block(&block_chain, block_template)
//                 .unwrap();
//
//             chain.clone().try_connect(block).await.unwrap();
//         }
//     }
//
//     (chain, node_config, storage)
// }
//
// #[stest::test(timeout = 480)]
// async fn test_block_chain_head() {
//     ::logger::init_for_test();
//     let times = 10;
//     let (chain, _, _) = gen_master_chain(times).await;
//     assert_eq!(
//         chain.master_head_header().await.unwrap().unwrap().number(),
//         times
//     );
// }
//
// #[stest::test(timeout = 480)]
// async fn test_block_chain_forks() {
//     ::logger::init_for_test();
//     let times = 5;
//     let (chain_service, config, storage) = gen_master_chain(times).await;
//     let mut parent_hash = chain_service
//         .clone()
//         .master_block_by_number(0)
//         .await
//         .unwrap()
//         .id();
//     let miner_account = AccountInfo::random();
//     if times > 0 {
//         for i in 0..(times + 1) {
//             //Delay::new(Duration::from_millis(1000)).await;
//
//             let chain =
//                 BlockChain::new(config.net().consensus(), parent_hash, storage.clone()).unwrap();
//             let (block_template, _) = chain
//                 .create_block_template(
//                     *miner_account.address(),
//                     Some(miner_account.get_auth_key().prefix().to_vec()),
//                     Some(parent_hash),
//                     Vec::new(),
//                     Vec::new(),
//                     None,
//                 )
//                 .unwrap();
//             let block = config
//                 .net()
//                 .consensus()
//                 .create_block(&chain, block_template)
//                 .unwrap();
//             // .into_block(0, 10000.into());
//             debug!(
//                 "{}:{:?}:{:?}:{:?}",
//                 i,
//                 parent_hash,
//                 block.header().id(),
//                 block.header().parent_hash()
//             );
//             parent_hash = block.header().id();
//             let _ = chain_service.clone().try_connect(block).await.unwrap();
//         }
//     }
//
//     assert_eq!(
//         chain_service
//             .master_head_header()
//             .await
//             .unwrap()
//             .unwrap()
//             .id(),
//         parent_hash
//     )
// }
//
// #[stest::test(timeout = 480)]
// ///             ╭--> b3(t2)
// /// Genesis--> b1--> b2(t2)
// ///
// async fn test_block_chain_txn_info_fork_mapping() -> Result<()> {
//     let config = Arc::new(NodeConfig::random_for_test());
//     let mut block_chain = test_helper::gen_blockchain_for_test(config.net())?;
//     let header = block_chain.current_header();
//     let miner_account = AccountInfo::random();
//
//     let (template_b1, _) = block_chain.create_block_template(
//         *miner_account.address(),
//         Some(miner_account.get_auth_key().prefix().to_vec()),
//         Some(header.id()),
//         vec![],
//         vec![],
//         None,
//     )?;
//
//     let block_b1 = config
//         .net()
//         .consensus()
//         .create_block(&block_chain, template_b1)?;
//     block_chain.apply(block_b1.clone())?;
//
//     let mut block_chain2 = block_chain.new_chain(block_b1.id()).unwrap();
//
//     // create transaction
//     let pri_key = Ed25519PrivateKey::genesis();
//     let public_key = pri_key.public_key();
//     let account_address = account_address::from_public_key(&public_key);
//     let signed_txn_t2 = {
//         let auth_prefix = AuthenticationKey::ed25519(&public_key).prefix().to_vec();
//         let txn = executor::build_transfer_from_association(
//             account_address,
//             auth_prefix,
//             0,
//             10000,
//             config.net().consensus().now() + 40000,
//             config.net(),
//         );
//         txn.as_signed_user_txn()?.clone()
//     };
//     let tnx_hash = signed_txn_t2.crypto_hash();
//     let (template_b2, _) = block_chain.create_block_template(
//         *miner_account.address(),
//         Some(miner_account.get_auth_key().prefix().to_vec()),
//         Some(block_b1.id()),
//         vec![signed_txn_t2.clone()],
//         vec![],
//         None,
//     )?;
//     let block_b2 = config
//         .net()
//         .consensus()
//         .create_block(&block_chain, template_b2)?;
//
//     block_chain.apply(block_b2)?;
//     let (template_b3, _) = block_chain2.create_block_template(
//         *miner_account.address(),
//         Some(miner_account.get_auth_key().prefix().to_vec()),
//         Some(block_b1.id()),
//         vec![signed_txn_t2],
//         vec![],
//         None,
//     )?;
//     let block_b3 = config
//         .net()
//         .consensus()
//         .create_block(&block_chain2, template_b3)?;
//     block_chain2.apply(block_b3)?;
//
//     let vec_txn = block_chain2
//         .get_storage()
//         .get_transaction_info_ids_by_hash(tnx_hash)?;
//
//     assert_eq!(vec_txn.len(), 2);
//     let txn_info = block_chain.get_transaction_info(tnx_hash)?;
//     assert!(txn_info.is_some());
//     assert_eq!(txn_info.unwrap().transaction_hash(), tnx_hash);
//     Ok(())
// }
//
// #[stest::test(timeout = 480)]
// async fn test_chain_apply() -> Result<()> {
//     let config = Arc::new(NodeConfig::random_for_test());
//     let mut block_chain = test_helper::gen_blockchain_for_test(config.net())?;
//     let header = block_chain.current_header();
//     debug!("genesis header: {:?}", header);
//
//     let miner_account = AccountInfo::random();
//     let (block_template, _) = block_chain.create_block_template(
//         *miner_account.address(),
//         Some(miner_account.get_auth_key().prefix().to_vec()),
//         Some(header.id()),
//         vec![],
//         vec![],0
//         None,
//     )?;
//
//     let new_block = config
//         .net()
//         .consensus()
//         .create_block(&block_chain, block_template)?;
//
//     // block_chain.txn_accumulator.append(&[HashValue::random()])?;
//     // block_chain.txn_accumulator.flush()?;
//     //
//     // block_chain.block_accumulator.append(&[new_block.id()])?;
//     // block_chain.block_accumulator.flush()?;
//     block_chain.apply(new_block)?;
//     // let header1 = block_chain.current_header();
//     // debug!("block 1 header: {:?}", header1);
//     // assert_ne!(header.state_root(), header1.state_root());
//     Ok(())
// }
