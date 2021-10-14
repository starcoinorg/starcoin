use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::Result;
use async_std::sync::Arc;
use logger::prelude::*;
use network::NetworkServiceRef;
use network_api::messages::PeerAnnouncementMessage;
use network_api::{PeerProvider, PeerSelector, PeerStrategy, ReputationChange};
use starcoin_crypto::HashValue;
use starcoin_network_rpc_api::GetTxnsWithHash;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{BlockTransactionInfoStore, Storage};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::transaction::TransactionError;
use txpool::TxPoolService;

/// Service which handle Announcement message
pub struct AnnouncementService {
    storage: Arc<Storage>,
    txpool: TxPoolService,
}

impl AnnouncementService {
    fn new(storage: Arc<Storage>, txpool: TxPoolService) -> Self {
        AnnouncementService { storage, txpool }
    }
}

impl ActorService for AnnouncementService {}

impl ServiceFactory<Self> for AnnouncementService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<AnnouncementService> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let txpool_service = ctx.get_shared::<TxPoolService>()?;

        Ok(Self::new(storage, txpool_service))
    }
}

impl EventHandler<Self, PeerAnnouncementMessage> for AnnouncementService {
    fn handle_event(
        &mut self,
        announcement_msg: PeerAnnouncementMessage,
        ctx: &mut ServiceContext<AnnouncementService>,
    ) {
        let txpool = self.txpool.clone();
        let storage = self.storage.clone();
        let network = ctx
            .get_shared::<NetworkServiceRef>()
            .expect("NetworkServiceRef not exist.");
        let peer_id = announcement_msg.peer_id.clone();
        debug!("[sync] receive announcement msg : {:?}", announcement_msg);

        ctx.spawn(async move {
            if announcement_msg.message.is_txn() {
                let fresh_ids = announcement_msg.message.ids().into_iter().filter(|txn_id| {
                    if txpool.find_txn(txn_id).is_none() {
                        if let Ok(None) = storage.get_transaction_info(*txn_id) {
                            return true
                        }
                    }
                    false
                }).collect::<Vec<HashValue>>();

                if !fresh_ids.is_empty() {
                    let peer_selector =
                        PeerSelector::new(Vec::new(), PeerStrategy::default(), None);
                    let rpc_client = VerifiedRpcClient::new(
                        peer_selector,
                        network.clone(),
                    );
                    match rpc_client.get_txns_with_hash_from_pool(Some(peer_id.clone()), GetTxnsWithHash { ids:fresh_ids }).await {
                        Err(err) => error!(
                            "[sync] handle announcement msg result error: {:?}, peer_id:{:?} ",
                            err, peer_id
                        ),
                        Ok((_, txns)) => {
                            let mut fresh_txns = Vec::new();
                            txns.into_iter().for_each(|txn| {
                                match txpool.verify_transaction(txn.clone()) {
                                    Ok(_) => fresh_txns.push(txn),
                                    Err(err) => {
                                        error!(
                                            "[sync] handle announcement msg error: {:?}, peer_id:{:?} ",
                                            err, peer_id
                                        );
                                        if let TransactionError::InvalidSignature(_) = err {
                                            network.report_peer(peer_id.clone(), ReputationChange::new(i32::min_value() / 2, "InvalidSignature"))
                                        }
                                    }
                                }
                            });

                            if !fresh_txns.is_empty() {
                                txpool.add_txns(fresh_txns);
                            }
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::NodeConfig;
    use futures::executor::block_on;
    use network_api::messages::TXN_PROTOCOL_NAME;
    use network_api::MultiaddrWithPeerId;
    use starcoin_txpool_api::TxPoolSyncService;
    use std::time::Duration;

    #[stest::test]
    fn test_get_txns_with_hash_from_pool() {
        let mut config_1 = NodeConfig::random_for_test();
        config_1.miner.disable_miner_client = Some(true);
        let config_1 = Arc::new(config_1);
        let service1 = test_helper::run_node_by_config(config_1.clone()).unwrap();

        std::thread::sleep(Duration::from_secs(1));
        let network1 = service1.network();
        let peer_1 = block_on(async { network1.get_self_peer().await.unwrap().peer_id() });
        let network1 = service1.network();
        let nodes = vec![MultiaddrWithPeerId::new(
            config_1.network.listen(),
            peer_1.clone().into(),
        )];
        let mut config_2 = NodeConfig::random_for_test();
        config_2.network.seeds = nodes.into();
        config_2.network.unsupported_protocols = Some(vec![TXN_PROTOCOL_NAME.to_string()]);
        config_2.miner.disable_miner_client = Some(true);
        let config_2 = Arc::new(config_2);
        let service2 = test_helper::run_node_by_config(config_2).unwrap();

        std::thread::sleep(Duration::from_secs(2));
        let network2 = service2.network();
        block_on(async move {
            let peer_2 = network2.get_self_peer().await.unwrap().peer_id();
            assert!(network2.is_connected(peer_1).await);
            assert!(network1.is_connected(peer_2).await);
        });
        let txpool = service1.txpool();
        let txns = test_helper::txn::create_account(config_1.net(), 0, 1);
        txpool
            .add_txns(txns.into_iter().map(|(_, txn)| txn).collect())
            .into_iter()
            .for_each(|r| r.unwrap());

        std::thread::sleep(Duration::from_secs(5));
        assert_eq!(service2.txpool().status().txn_count, 1);
    }
}
