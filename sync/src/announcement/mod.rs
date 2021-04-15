use crate::verified_rpc_client::VerifiedRpcClient;
use anyhow::Result;
use async_std::sync::Arc;
use logger::prelude::*;
use network::NetworkServiceRef;
use network_api::messages::{Announcement, PeerAnnouncementMessage};
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
    network: NetworkServiceRef,
}

impl AnnouncementService {
    fn new(storage: Arc<Storage>, txpool: TxPoolService, network: NetworkServiceRef) -> Self {
        AnnouncementService {
            storage,
            txpool,
            network,
        }
    }
}

impl ActorService for AnnouncementService {}

impl ServiceFactory<Self> for AnnouncementService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<AnnouncementService> {
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let txpool_service = ctx.get_shared::<TxPoolService>()?;
        let network = ctx.get_shared::<NetworkServiceRef>()?;

        Ok(Self::new(storage, txpool_service, network))
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
        let network = self.network.clone();
        let peer_id = announcement_msg.peer_id.clone();
        ctx.spawn(async move {
            match announcement_msg.message {
                Announcement::TXN(ids) => {
                    let fresh_ids = ids.into_iter().filter(|txn_id| {
                        if txpool.find_txn(&txn_id).is_none() {
                            if let Ok(None) = storage.get_transaction_info(txn_id.clone()) {
                                return true
                            }
                        }
                        false
                    }).collect::<Vec<HashValue>>();

                    if !fresh_ids.is_empty() {
                        let peer_selector =
                            PeerSelector::new(Vec::new(), PeerStrategy::default());
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
            }
        });
    }
}
