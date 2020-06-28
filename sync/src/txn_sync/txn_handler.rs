use crate::helper::do_response_get_txns;
use anyhow::Result;
use futures::channel::mpsc;
use starcoin_storage::Store;
use starcoin_sync_api::{GetTxns, TransactionsData};
use starcoin_txpool_api::TxPoolSyncService;
use std::borrow::Cow;
use std::sync::Arc;
use txpool::TxPoolService;

#[derive(Clone)]
pub struct GetTxnsHandler {
    pool: TxPoolService,
    storage: Arc<dyn Store>,
}

impl GetTxnsHandler {
    pub fn new(txpool: TxPoolService, storage: Arc<dyn Store>) -> GetTxnsHandler {
        Self {
            pool: txpool,
            storage,
        }
    }
}

// TODO: we can do more logic here
impl GetTxnsHandler {
    pub async fn handle(
        self,
        responder: mpsc::Sender<(Cow<'static, [u8]>, Vec<u8>)>,
        msg: GetTxns,
    ) -> Result<()> {
        let data = {
            match msg.ids {
                // get from txpool
                None => self.pool.get_pending_txns(None),
                // get from storage
                Some(ids) => {
                    let mut data = vec![];
                    for id in ids {
                        if let Some(txn) = self.storage.get_transaction(id)? {
                            data.push(txn.as_signed_user_txn()?.clone());
                        }
                    }
                    data
                }
            }
        };
        do_response_get_txns(responder, TransactionsData { txns: data }).await
    }
}
